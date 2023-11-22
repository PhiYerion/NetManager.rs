pub struct AddressFamily {}
#[allow(non_upper_case_globals)]
/// Linux address families.
/// See [https://github.com/torvalds/linux/blob/master/include/linux/socket.h]
impl AddressFamily {
    pub const UNSPEC: u8 = 0;
    pub const LOCAL: u8 = 1;
    pub const UNIX: u8 = 1;
    pub const FILE: u8 = 1;
    pub const INET: u8 = 2;
    pub const AX25: u8 = 3;
    pub const IPX: u8 = 4;
    pub const APPLETALK: u8 = 5;
    pub const NETROM: u8 = 6;
    pub const BRIDGE: u8 = 7;
    pub const ATMPVC: u8 = 8;
    pub const X25: u8 = 9;
    pub const INET6: u8 = 10;
    pub const ROSE: u8 = 11;
    pub const DECnet: u8 = 12;
    pub const NETBEUI: u8 = 13;
    pub const SECURITY: u8 = 14;
    pub const KEY: u8 = 15;
    pub const NETLINK: u8 = 16;
    pub const ROUTE: u8 = 16;
    pub const PACKET: u8 = 17;
    pub const ASH: u8 = 18;
    pub const ECONET: u8 = 19;
    pub const ATMSVC: u8 = 20;
    pub const RDS: u8 = 21;
    pub const SNA: u8 = 22;
    pub const IRDA: u8 = 23;
    pub const PPPOX: u8 = 24;
    pub const WANPIPE: u8 = 25;
    pub const LLC: u8 = 26;
    pub const IB: u8 = 27;
    pub const MPLS: u8 = 28;
    pub const CAN: u8 = 29;
    pub const TIPC: u8 = 30;
    pub const BLUETOOTH: u8 = 31;
    pub const IUCV: u8 = 32;
    pub const RXRPC: u8 = 33;
    pub const ISDN: u8 = 34;
    pub const PHONET: u8 = 35;
    pub const IEEE802154: u8 = 36;
    pub const CAIF: u8 = 37;
    pub const ALG: u8 = 38;
    pub const NFC: u8 = 39;
    pub const VSOCK: u8 = 40;
    pub const KCM: u8 = 41;
    pub const QIPCRTR: u8 = 42;
    pub const SMC: u8 = 43;
    pub const XDP: u8 = 44;
    pub const MCTP: u8 = 45;
    pub const MAX: u8 = 46;
}
