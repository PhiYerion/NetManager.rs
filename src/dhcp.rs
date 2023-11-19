// Build the DHCP Discover packet
use pnet::datalink::NetworkInterface;

use pnet::packet::{
    ethernet::{EtherTypes, MutableEthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::MutableIpv4Packet,
    udp::MutableUdpPacket,
    Packet,
};
use pnet::util::MacAddr;
use rand::Rng;
use std::net::Ipv4Addr;

pub const DHCP_PACKET_LEN: usize = 314;

pub fn remove_trailing_zeros(mut packet: Vec<u8>) -> Vec<u8> {
    assert!(!packet.is_empty());
    // This will return the last non-zero byte index
    let last_nonzero = packet.iter().rposition(|&r| r != 0);

    let padding_start = match last_nonzero {
        // We don't want to remove the last non-zero byte
        Some(x) => x + 1,
        // There should be some non-zero bytes
        None => panic!("There packet only contains 0s"),
    };

    packet.truncate(padding_start);
    packet
}

pub fn build_dhcp_to_layer2(
    dhcp_packet: Vec<u8>,
    interface: &NetworkInterface,
) -> Result<MutableEthernetPacket, String> {
    // We don't have a source addr yet, so we zero out this.'
    let source_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
    // We don't know where the router is, so we will broadcast this message'
    let destination_ipv4 = Ipv4Addr::new(255, 255, 255, 255);

    // UDP packet
    //   pnet requires &[u8], and this keeps this on the stack. DHCP packets shouldn't exceed 322
    //   bytes. 322 bytes is small enough to where it should be on the stack.
    let mut base_buffer = [0; DHCP_PACKET_LEN + 8];
    let mut buffer = match base_buffer.get(0..dhcp_packet.len()) {
        Some(mut x) => &mut x,
        None => {
            panic!("noooo");
            return Err("DHCP packet length (".to_string()
                + &dhcp_packet.len().to_string().as_str()
                + ") is more than allowable DHCP packet length ("
                + &DHCP_PACKET_LEN.to_string().as_str()
                + ")");
        }
    };
    let comparison_copy = dhcp_packet.clone();
    let mut udp_packet = MutableUdpPacket::new(buffer).unwrap();
    {
        // Header
        udp_packet.set_source(68);
        udp_packet.set_destination(67);
        udp_packet.set_length(DHCP_PACKET_LEN as u16);

        // Payload
        udp_packet.set_payload(buffer);
    }
    debug_assert_eq!(
        remove_trailing_zeros(udp_packet.payload().to_vec()),
        comparison_copy
    );

    // IPv4 packet
    let mut padding2: [u8; DHCP_PACKET_LEN + 28] = [0; DHCP_PACKET_LEN + 28];
    let mut ipv4_packet = MutableIpv4Packet::new(&mut padding2).unwrap();
    {
        // Header:
        ipv4_packet.set_version(4);
        ipv4_packet.set_header_length(5);
        ipv4_packet.set_identification(rand::thread_rng().gen::<u16>()); // Will not use this later (yet)
        ipv4_packet.set_source(source_ipv4);
        ipv4_packet.set_destination(destination_ipv4);
        ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
        ipv4_packet.set_ttl(64);
        ipv4_packet.set_total_length((DHCP_PACKET_LEN + 28) as u16);

        // Check sum:
        let checksum_value = pnet::packet::ipv4::checksum(&ipv4_packet.to_immutable());
        ipv4_packet.set_checksum(checksum_value);

        // Payload:
        ipv4_packet.set_payload(udp_packet.packet());
    }
    assert_eq!(udp_packet.packet(), ipv4_packet.payload());

    let ethernet_buffer = vec![0u8; DHCP_PACKET_LEN + 42];

    let mut ethernet_packet = MutableEthernetPacket::owned(ethernet_buffer).unwrap();
    {
        ethernet_packet.set_destination(MacAddr::broadcast());
        ethernet_packet.set_source(interface.mac.unwrap());
        ethernet_packet.set_ethertype(EtherTypes::Ipv4);
        ethernet_packet.set_payload(ipv4_packet.packet());
    }
    assert_eq!(ethernet_packet.payload(), ipv4_packet.packet());

    Ok(ethernet_packet)
}
