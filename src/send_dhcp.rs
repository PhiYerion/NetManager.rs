use std::error::Error;
use std::fmt;

use std::net::{Ipv4Addr};
use netdevice::get_hardware;

use pnet::datalink::{self, Channel, DataLinkReceiver, NetworkInterface};
use pnet::packet::{FromPacket, Packet,};
use pnet::packet::ethernet::{EthernetPacket};
use pnet::packet::ipv4::{Ipv4Packet};
use pnet::packet::udp::{UdpPacket};
use pnet::packet::dhcp::{DhcpPacket, Dhcp, MutableDhcpPacket};
use pnet::util::{MacAddr};
use crate::dhcp::{CustDhcp, DhcpOptions};
use crate::dhcp::DHCP_PACKET_LEN;
use crate::mac::get_mac;

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


pub fn get_netmask<'a>(interface_name: &String) -> Result<Network, Box<dyn Error>> {
    let mac = get_mac(interface_name);
    let interface = match get_interface(interface_name) {
        Some(r) => r,
        None => return Err(Box::new(DhcpError::Specific("Unable to find interface".to_string())))
    };
    dbg!("Got interface {}", &interface.name);

    let dhcp_wrapper = CustDhcp::new(mac, DhcpOptions::Slim)?;
    let xid = dhcp_wrapper.packet.xid;
    dbg!("Built dhcp_wrapper");

    let eframe = &mut dhcp_wrapper.build_dhcp_to_layer2(&interface);
    dbg!("Built ethernet frame");

    let res = match get_dhcp_offer(
        xid,
        send_packet(&interface, eframe.to_immutable()),
        mac
        ) {
            Some(r) => r,
            None => return Err(Box::new(DhcpError::Specific("Unable create get dhcp response".to_string())))
        };

    dbg!("Got dhcp offer");

    let mut dhcp_packet = MutableDhcpPacket::owned(vec![0u8; DHCP_PACKET_LEN]).unwrap();

    MutableDhcpPacket::populate(&mut dhcp_packet, &res);

    Ok(format_dhcp_offer(dhcp_packet.to_immutable()))
}

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

fn send_packet (interface: &NetworkInterface, packet: EthernetPacket) -> Box<dyn DataLinkReceiver> {
    // Send the packet
    let (mut tx, rx) = match datalink::channel(&interface, Default::default()) {
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
            Some(packet) => {
                dbg!(&packet);
                packet
            },
            None => continue, // Skip packets that are not Ethernet
        };

        if ethernet_packet.get_destination() != mac {
            continue; // Skip packets with a different MAC address
        }

        let udp_packet = match UdpPacket::new(ethernet_packet.payload()) {
            Some(packet) => {
                dbg!(&packet);
                packet
            },
            None => continue, // Skip packets that are not UDP
        };

        let ipv4_packet = match Ipv4Packet::new(udp_packet.payload()) {
            Some(packet) => {
                dbg!(&packet);
                packet
            },
            None => continue, // Skip packets that are not IPv4
        };

        let dhcp_packet = match DhcpPacket::new(ipv4_packet.payload()) {
            Some(packet) => {
                dbg!(&packet);
                packet
            },
            None => continue, // Skip packets that are not DHCP
        };

        dbg!("Got a DHCP packet");
        if dhcp_packet.get_xid() == xid {
            return Some(dhcp_packet.from_packet());
        };
        dbg!("Incoming DHCP packet has wrong xid", dhcp_packet.get_xid(), xid);
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
