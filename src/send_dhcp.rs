use std::error::Error;
use std::fmt;
use std::net::{Ipv4Addr};

use pnet::datalink::{self, Channel, DataLinkReceiver, NetworkInterface};
use pnet::packet::{FromPacket, Packet,};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes, MutableEthernetPacket};
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::udp::{MutableUdpPacket, UdpPacket};
use pnet::packet::dhcp::{DhcpPacket, Dhcp, MutableDhcpPacket};
use pnet::util::{MacAddr};
use crate::dhcp::CustDhcp;
use crate::dhcp::DHCP_PACKET_LEN;

#[derive(Debug)]
enum DhcpError {
    Generic,
    Specific(String),
}

// Implement the std::error::Error trait for the custom error enum
impl Error for DhcpError {}

// Implement the fmt::Display trait to customize how the error is displayed
impl fmt::Display for DhcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DhcpError::Generic => write!(f, "An unspecified dhcp error occurred."),
            DhcpError::Specific(message) => write!(f, "A dhcp error occurred: {}", message),
        }
    }
}

#[derive(Debug)]
pub struct Network {
    options: [Option<Ipv4Addr>; 3]
}

impl Network {
    fn get_netmask(&self) -> Option<Ipv4Addr> {
        self.options[0]
    }
    fn get_gateway(&self) -> Option<Ipv4Addr> {
        self.options[1]
    }
    fn get_dns(&self) -> Option<Ipv4Addr> {
        self.options[2]
    }
}

pub fn get_netmask<'a>(interface_name: &str) -> Result<Network, Box<dyn Error>> {
    let interface = match get_interface(interface_name) {
        Some(r) => r,
        None => return Err(Box::new(DhcpError::Specific("Unable to find interface".to_string())))
    };

    let dhcp_wrapper = CustDhcp::new()?;
    let xid = dhcp_wrapper.xid;

    let eframe = &mut build_ethernet_frame(
        &interface,
        dhcp_wrapper
    );

    let res = match get_dhcp_offer(
        xid,
        send_packet(&interface, eframe.to_immutable()),
        MacAddr(0x18, 0xc0, 0x4d, 0x5b, 0x03, 0xae)
        ) {
            Some(r) => r,
            None => return Err(Box::new(DhcpError::Specific("Unable create get dhcp response".to_string())))
        };

    let mut dhcp_packet = MutableDhcpPacket::owned(vec![0u8; DHCP_PACKET_LEN]).unwrap();

    MutableDhcpPacket::populate(&mut dhcp_packet, &res);

    Ok(format_dhcp_offer(dhcp_packet.to_immutable()))
}

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

fn build_ethernet_frame(interface: &NetworkInterface, mut dhcp_wrapper: CustDhcp) -> MutableEthernetPacket {

    // Build the UDP packet
    let mut padding = [0; DHCP_PACKET_LEN + 8];
    let mut udp_packet = MutableUdpPacket::new(&mut padding).unwrap();

    udp_packet.set_source(68);
    udp_packet.set_destination(67);
    udp_packet.set_length(DHCP_PACKET_LEN as u16);
    udp_packet.set_payload(&dhcp_wrapper.packet);
    udp_packet.to_immutable();
    {
        let mut udp_header = MutableUdpPacket::new(&mut dhcp_wrapper.packet).unwrap();
        udp_header.set_source(68);
        udp_header.set_destination(67);
        udp_header.set_length(DHCP_PACKET_LEN as u16);
        let checksum = pnet::packet::udp::ipv4_checksum(
            &udp_header.to_immutable(),
            &Ipv4Addr::new(0, 0, 0, 0),
            &Ipv4Addr::new(255, 255, 255, 255));
        udp_header.set_checksum(checksum);
    }

    // Build the IPv4 packet
    let source_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
    let destination_ipv4 = Ipv4Addr::new(255, 255, 255, 255);

    let mut padding2: [u8; DHCP_PACKET_LEN + 28] = [0; DHCP_PACKET_LEN + 28];
    let mut ipv4_packet = MutableIpv4Packet::new(&mut padding2).unwrap();
    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    // CHANGE THIS
    ipv4_packet.set_identification(0xbb04);
    ipv4_packet.set_total_length((DHCP_PACKET_LEN + 28) as u16);
    ipv4_packet.set_payload(udp_packet.packet());
    ipv4_packet.set_source(source_ipv4);
    ipv4_packet.set_destination(destination_ipv4);
    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
    ipv4_packet.set_ttl(64);
    let checksum_value = pnet::packet::ipv4::checksum(&ipv4_packet.to_immutable());
    ipv4_packet.set_checksum(checksum_value);

    let ethernet_buffer = vec![0u8; DHCP_PACKET_LEN + 42];

    let mut ethernet_packet = MutableEthernetPacket::owned(ethernet_buffer).unwrap();
    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(interface.mac.unwrap());
    ethernet_packet.set_ethertype(EtherTypes::Ipv4);
    ethernet_packet.set_payload(ipv4_packet.packet());

    ethernet_packet
}

fn send_packet (interface: &NetworkInterface, packet: EthernetPacket) -> Box<dyn DataLinkReceiver> {
    // Send the packet
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error creating datalink channel: {}", e),
    };

    tx.send_to(packet.packet(), None);

    rx
}

fn get_dhcp_offer<'a>(
    xid: u32,
    mut rx: Box<dyn DataLinkReceiver>,
    mac: MacAddr,
) -> Option<Dhcp> {

    while let Ok(base_packet) = rx.next() {
        // Process the received packet
        let ethernet_packet = match EthernetPacket::new(base_packet) {
            Some(packet) => packet,
            None => continue, // Skip packets that are not Ethernet
        };

        if ethernet_packet.get_destination() != mac {
            continue; // Skip packets with a different MAC address
        }

        let udp_packet = match UdpPacket::new(ethernet_packet.payload()) {
            Some(packet) => packet,
            None => continue, // Skip packets that are not UDP
        };

        let ipv4_packet = match Ipv4Packet::new(udp_packet.payload()) {
            Some(packet) => packet,
            None => continue, // Skip packets that are not IPv4
        };

        let dhcp_packet = match DhcpPacket::new(ipv4_packet.payload()) {
            Some(packet) => packet,
            None => continue, // Skip packets that are not DHCP
        };

        if dhcp_packet.get_xid() == xid {
            let a = dhcp_packet.from_packet();
            return Some(a);
        };
    }
    None
}

fn ipv4_from_u8_array (array: &[u8; 4]) -> Ipv4Addr {
    Ipv4Addr::new(array[0], array[1], array[2], array[3])
}

fn format_dhcp_offer (dhcp_offer_packet: DhcpPacket) -> Network {
    let dhcp_offer = dhcp_offer_packet.from_packet();
    let mut index = 0;
    let options_data = dhcp_offer.options;
    let mut net: Network = Network { options: [None, None, None] };
    while index < options_data.len() - 1 {
        let code = options_data[index];
        if code == 0x63 {
            index += 4;
            continue;
        }

        use arrayref::{array_ref};
        if code == 0x01 {
            let start = index + 2;
            let netmask = array_ref!(options_data, start, 4);
            net.options[0] = Some(ipv4_from_u8_array(netmask));
        } else if code == 0x36 {
            let start = index + 2;
            let router = array_ref!(options_data, start, 4);
            net.options[1] = Some(ipv4_from_u8_array(router));
        } else if code == 0x06 {
            let start = index + 2;
            let dns = array_ref!(options_data, start, 4);
            net.options[2] = Some(ipv4_from_u8_array(dns));
        }

        let length = options_data[index + 1] as usize;
        index += length + 2;
    }
    net
}
