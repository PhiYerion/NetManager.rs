// Build the DHCP Discover packet
use rand::Rng;
use std::net::{Ipv4Addr};
use pnet::packet::dhcp::DhcpHardwareTypes::Ethernet;
use pnet::packet::dhcp::{Dhcp, MutableDhcpPacket, DhcpOperation};
use pnet::packet::Packet;
use pnet::util::MacAddr;

pub const DHCP_PACKET_LEN: usize = 314;

pub struct CustDhcp {
    pub xid: u32,
    pub packet: [u8; DHCP_PACKET_LEN],
}

impl CustDhcp {
    pub fn new () -> Result<Self, Box<dyn std::error::Error>> {
        let transaction_id = rand::thread_rng().gen::<u32>();

        let dhcp_request = Dhcp {
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
            chaddr: MacAddr::new(0x18, 0xc0, 0x4d, 0x5b, 0x03, 0xae),
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
                            0xff, 0x4d, 0x5b, 0x03, 0xae,       // IAID
                            0x00, 0x01, 0x00, 0x01,             // Misc
                            0x2c, 0x52, 0xed, 0x4c,             // Time (yeah this is problematic)
                            0x18, 0xc0, 0x4d, 0x5b, 0x03, 0xae, // Link Layer Address
                          0x50, 0x00,                       // Rapid Commit
                          0x91, 0x01, 0x01,                 // Forcerenew Nonce Capable
                          0xff                              // End
            ],
        };

        let mut dhcp_packet = MutableDhcpPacket::owned(vec![0u8; DHCP_PACKET_LEN]).expect(" ");
        MutableDhcpPacket::populate(&mut dhcp_packet, &dhcp_request);
        Ok(Self {
            xid: transaction_id,
            packet: dhcp_packet.packet().try_into()?,
        })
    }
}
