use std::error::Error;
use std::net::Ipv4Addr;
use libc::{AF_INET, sa_family_t, sockaddr};
use default_net::interface::get_interfaces;

pub fn gen_sockaddr(addr: Ipv4Addr) -> sockaddr {
    sockaddr {
        sa_family: AF_INET as sa_family_t,
        sa_data: [0, 0,
            addr.octets()[0] as i8,
            addr.octets()[1] as i8,
            addr.octets()[2] as i8,
            addr.octets()[3] as i8,
            0, 0, 0, 0, 0, 0, 0, 0]
    }
}

pub fn get_interface_names() -> Vec<String> {
    get_interfaces()
        .iter()
        .map(|interface| interface.name.clone())
        .collect()
}