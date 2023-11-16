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

pub fn build_dhcp_to_layer2(
    dhcp_packet: Vec<u8>,
    interface: &NetworkInterface,
) -> MutableEthernetPacket {
    let source_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
    let destination_ipv4 = Ipv4Addr::new(255, 255, 255, 255);

    // UDP packet
    let mut padding = [0; DHCP_PACKET_LEN + 8];
    let comparison_copy = dhcp_packet.clone();
    let mut udp_packet = MutableUdpPacket::new(&mut padding).unwrap();
    {
        // Header
        udp_packet.set_source(68);
        udp_packet.set_destination(67);
        udp_packet.set_length(DHCP_PACKET_LEN as u16);

        // Payload
        udp_packet.set_payload(&dhcp_packet);
    }
    //assert_eq!(comparison_copy, udp_packet.payload());

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

    ethernet_packet
}
