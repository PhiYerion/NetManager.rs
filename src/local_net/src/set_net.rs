use std::io::Error;
use std::net::Ipv4Addr;
use std::process::Command;
use libc::{AF_INET, c_int, sa_family_t, sockaddr};
use netdevice::{set_address, set_destination};
use net_route::{Route, Handle};
use pnet::datalink;
use std::io;


macro_rules! cmd {
    ($name:literal, $command:expr) => {{
        let output = std::process::Command::new("ip")
            .args(&$command)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    Ok(result.stdout)
                } else {
                    Err(String::from_utf8_lossy(&result.stderr).to_string())
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }};
}

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

pub async fn up(interface: &str, addr: Ipv4Addr) -> Result<u8, Error> {
    let output: std::process::Output = Command::new("ip")
        .args(&["link", "set", "up", interface])
        .output()
        .expect("Failed to execute 'ip set up' command");

    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip set up': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let new_address = sockaddr{
        sa_family: AF_INET as sa_family_t,
        sa_data: [0, 0,
            addr.octets()[0] as i8,
            addr.octets()[1] as i8,
            addr.octets()[2] as i8,
            addr.octets()[3] as i8,
            0, 0, 0, 0, 0, 0, 0, 0]
    };

    set_address(new_socket()?, interface, new_address);
    set_destination(new_socket()?, interface, new_address);

    let handle = Handle::new()?;
    let interface_index = || {
        for interface_iter in datalink::interfaces() {
            if interface_iter.name == interface {
                return Some(interface_iter.index)
            }
        }
        None
    };

    let route = Route::new("192.168.4.0".parse().unwrap(), 22)
        .with_ifindex(interface_index().unwrap())
        .with_gateway("192.168.4.1".parse().unwrap());

    dbg!(handle.add(&route).await.unwrap());

    Ok(1)
}

pub fn down(interface: &str) {

    // ip address flush dev enp7s0
    let output: std::process::Output = Command::new("ip")
    .args(&["address", "flush", "dev", interface])
    .output()
    .expect("Failed to execute 'ip address flush' command");

    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip address flush': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // ip route flush dev enp7s0
    let output = Command::new("ip")
        .args(&["route", "flush", "dev", interface])
        .output()
        .expect("Failed to execute 'ip route flush' command");

    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip route flush': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // ip link set down enp7s0
    let output = Command::new("ip")
        .args(&["link", "set", "down", interface])
        .output()
        .expect("Failed to execute 'ip link set down' command");


    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip link set down': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}