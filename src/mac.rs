use netdevice::{get_hardware, set_hardware};
use libc::{ARPHRD_ETHER, c_int, c_char, sockaddr};
use pnet::util::MacAddr;
use std::io::Error;

pub fn new_socket() -> Result<c_int, Error> {
    use libc::{AF_INET,
               IPPROTO_UDP,
               SOCK_DGRAM};

    let res = unsafe { libc::socket(
        AF_INET,
        SOCK_DGRAM,
        IPPROTO_UDP) };

    match res {
        -1 => Err(Error::last_os_error()),
        sock => Ok(sock),
    }
}


pub fn set_mac (interface: &String, mac: MacAddr) {

    let mut old = get_hardware(
        new_socket().unwrap(), interface).unwrap().sa_data;

    let mut new_addr: [u8; 6] = [mac.0, mac.1, mac.2, mac.3, mac.4, mac.5];
    new_addr[0] &= 0xfc;
    new_addr[0] |= 0x02;
    for i in 0..6 {
        old[i] = new_addr[i] as c_char;
    }

    let sock = sockaddr { sa_family: ARPHRD_ETHER, sa_data: old };

    set_hardware(new_socket().unwrap(), interface, sock).unwrap();
}

pub fn get_mac (interface: &String) -> MacAddr {
    let res = get_hardware(new_socket().unwrap(), interface).unwrap();
    let raw_mac = res.sa_data;
    MacAddr::new(raw_mac[0] as u8, raw_mac[1] as u8, raw_mac[2] as u8,
                 raw_mac[3] as u8, raw_mac[4] as u8, raw_mac[5] as u8)
}