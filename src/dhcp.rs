// Build the DHCP Discover packet
use rand::Rng;
use std::net::Ipv4Addr;
use pnet::datalink::NetworkInterface;
use pnet::util::MacAddr;
use pnet::packet::{
    Packet,
    ip::IpNextHeaderProtocols,
    ipv4::MutableIpv4Packet,
    udp::MutableUdpPacket,
    dhcp::{
        Dhcp,
        MutableDhcpPacket,
        DhcpOperation,
        DhcpHardwareTypes::Ethernet
    },
    ethernet::{EtherTypes, MutableEthernetPacket},

};

pub const DHCP_PACKET_LEN: usize = 314;

pub struct CustDhcp {
    pub packet: Dhcp
}

pub enum DhcpOptions {
    NetmaskSlim,
    Slim,
    Full
}

impl CustDhcp {
    pub fn new (mac: MacAddr, dhcp_options: DhcpOptions) -> Result<Self, Box<dyn std::error::Error>> {
        let transaction_id = rand::thread_rng().gen::<u32>();
        dbg!(transaction_id);

        let packet = match dhcp_options {
            DhcpOptions::Full => {
                 Dhcp {
                    op: DhcpOperation { 0: 1 },                     // BOOTREQUEST
                    htype: Ethernet,                                // Ethernet
                    hlen: 6,                                        // MAC address length
                    hops: 0,
                    xid: transaction_id,
                    secs: 0,
                    flags: 0x0000,                                  // Unicast flag?
                    ciaddr: Ipv4Addr::new(0, 0, 0, 0),
                    yiaddr: Ipv4Addr::new(0, 0, 0, 0),
                    siaddr: Ipv4Addr::new(0, 0, 0, 0),
                    giaddr: Ipv4Addr::new(0, 0, 0, 0),
                    chaddr: mac,
                    chaddr_pad: vec![],
                    sname: vec![],
                    file: vec![],
                    options: vec![0x63, 0x82, 0x53, 0x63,           // Magic cookie: DHCP
                                  0x35, 0x01, 0x01,                 // DHCP Discover
                                  0x37, 0x0e,                       // Parameter List:
                                  0x01,                               // Subnet Mask
                                  0x79,                               // Classless Static Route
                                  0x03,                               // Router
                                  0x06,                               // Domain Name Server
                                  0x0c,                               // Host Name
                                  0x0f,                               // Domain Name
                                  0x1a,                               // Interface MTU
                                  0x1c,                               // Broadcast Address
                                  0x21,                               // Static Route
                                  0x33,                               // IP Address Lease Time
                                  0x36,                               // DHCP Server Identifier
                                  0x3a,                               // Renewal Time Value
                                  0x3b,                               // Rebinding Time Value
                                  0x77,                               // Domain Search
                                  0x39, 0x02, 0x05, 0xc0,           // Max DHCP Message Size
                                  0x3d, 0x13,                       // Client identifier
                                  0xff, mac.2, mac.3, mac.4, mac.5,   // IAID
                                  0x00, 0x01, 0x00, 0x01,             // Misc
                                  0x2c, 0x52, 0xed, 0x4c,             // Time (yeah this is problematic)
                                  mac.0, mac.1, mac.2, mac.3, mac.4, mac.5, // Link Layer Address
                                  0x50, 0x00,                       // Rapid Commit
                                  0x91, 0x01, 0x01,                 // Forcerenew Nonce Capable
                                  0xff                              // End
                    ],
                }
            }
            DhcpOptions::Slim => {
                Dhcp {
                    op: DhcpOperation { 0: 1 },                     // BOOTREQUEST
                    htype: Ethernet,                                // Ethernet
                    hlen: 6,                                        // MAC address length
                    hops: 0,
                    xid: transaction_id,
                    secs: 0,
                    flags: 0x0000,                                  // Unicast flag?
                    ciaddr: Ipv4Addr::new(0, 0, 0, 0),
                    yiaddr: Ipv4Addr::new(0, 0, 0, 0),
                    siaddr: Ipv4Addr::new(0, 0, 0, 0),
                    giaddr: Ipv4Addr::new(0, 0, 0, 0),
                    chaddr: mac,
                    chaddr_pad: vec![],
                    sname: vec![],
                    file: vec![],
                    options: vec![0x63, 0x82, 0x53, 0x63,           // Magic cookie: DHCP
                                  0x35, 0x01, 0x01,                 // DHCP Discover
                                  0x37, 0x0e,                       // Parameter List:
                                  0x01,                               // Subnet Mask
                                  0x03,                               // Router
                                  0x21,                               // Static Route
                                  0xff                              // End
                    ],
                }
            }
            DhcpOptions::NetmaskSlim => {
                Dhcp {
                    op: DhcpOperation { 0: 1 },                     // BOOTREQUEST
                    htype: Ethernet,                                // Ethernet
                    hlen: 6,                                        // MAC address length
                    hops: 0,
                    xid: transaction_id,
                    secs: 0,
                    flags: 0x0000,                                  // Unicast flag?
                    ciaddr: Ipv4Addr::new(0, 0, 0, 0),
                    yiaddr: Ipv4Addr::new(0, 0, 0, 0),
                    siaddr: Ipv4Addr::new(0, 0, 0, 0),
                    giaddr: Ipv4Addr::new(0, 0, 0, 0),
                    chaddr: mac,
                    chaddr_pad: vec![],
                    sname: vec![],
                    file: vec![],
                    options: vec![0x63, 0x82, 0x53, 0x63,           // Magic cookie: DHCP
                                  0x35, 0x01, 0x01,                 // DHCP Discover
                                  0x37, 0x0e,                       // Parameter List:
                                  0x01,                               // Subnet Mask
                                  0xff                              // End
                    ],
                }
            }
        };

        Ok(CustDhcp { packet })
    }

    fn get_raw_packet(&self) -> [u8; 314] {
        let mut dhcp_packet = MutableDhcpPacket::owned(vec![0u8; DHCP_PACKET_LEN]).expect(" ");
        dhcp_packet.populate(&self.packet);
        let mut raw_packet: [u8; DHCP_PACKET_LEN] = [0; DHCP_PACKET_LEN];
        raw_packet.copy_from_slice(dhcp_packet.packet());
        raw_packet
    }

    pub fn build_dhcp_to_layer2(&self, interface: &NetworkInterface) -> MutableEthernetPacket {
        let source_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
        let destination_ipv4 = Ipv4Addr::new(255, 255, 255, 255);

        // UDP packet
        let mut padding = [0; DHCP_PACKET_LEN + 8];
        let comparison_copy = self.get_raw_packet();
        let mut udp_packet = MutableUdpPacket::new(&mut padding).unwrap();
        {
            // Header
            udp_packet.set_source(68);
            udp_packet.set_destination(67);
            udp_packet.set_length(DHCP_PACKET_LEN as u16);

            // Payload
            udp_packet.set_payload(&self.get_raw_packet());
        }
        assert_eq!(comparison_copy, udp_packet.payload());

        // IPv4 packet
        let mut padding2: [u8; DHCP_PACKET_LEN + 28] = [0; DHCP_PACKET_LEN + 28];
        let mut ipv4_packet = MutableIpv4Packet::new(&mut padding2).unwrap();
        {
            // Header:
            ipv4_packet.set_version(4);
            ipv4_packet.set_header_length(5);
            ipv4_packet.set_identification(rand::thread_rng().gen::<u16>());                                // Will not use this later (yet)
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
}
