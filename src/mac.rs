use libc::{c_char, c_int, sockaddr, ARPHRD_ETHER};
use netdevice::{get_hardware, set_hardware};
use pnet::util::MacAddr;
use std::io::Error;

pub fn new_socket() -> Result<c_int, Error> {
    use libc::{AF_INET, IPPROTO_UDP, SOCK_DGRAM};

    let res = unsafe { libc::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP) };

    match res {
        -1 => Err(Error::last_os_error()),
        sock => Ok(sock),
    }
}

// This should really have some unit tests, but without a docker
// container it would be weird.
pub fn set_mac(interface: &str, mac: MacAddr) {
    let mut old = get_hardware(new_socket().unwrap(), interface)
        .unwrap()
        .sa_data;

    let mut new_addr: [u8; 6] = [mac.0, mac.1, mac.2, mac.3, mac.4, mac.5];
    new_addr[0] &= 0xfc;
    new_addr[0] |= 0x02;
    for i in 0..6 {
        old[i] = new_addr[i] as c_char;
    }

    let sock = sockaddr {
        sa_family: ARPHRD_ETHER,
        sa_data: old,
    };

    set_hardware(new_socket().unwrap(), interface, sock).unwrap();
}

pub fn get_mac(interface: &str) -> [u8; 14] {
    let res = get_hardware(new_socket().unwrap(), interface).unwrap();

    // i8 -> u8 is very safe. I hope to god the compiler
    // doesn't actually preform any additional computation.
    unsafe { std::mem::transmute(res.sa_data) }
}
