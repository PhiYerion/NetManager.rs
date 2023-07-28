use std::io::Error;
use std::net::Ipv4Addr;
use std::process::Command;
use libc::{c_char, c_int, sockaddr};
use netdevice::{set_address, set_destination, set_broadcast, set_netmask, get_address};

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

pub fn up(interface: &str, addr: Ipv4Addr) -> Result<u8, Error> {
    let mut sock = get_address(new_socket()?, interface)?;
    dbg!(&sock.sa_data);
    for i in 2..6 {
        sock.sa_data[i] = addr.octets()[i - 2] as c_char;
    }
    dbg!(&sock.sa_data);

    set_address(new_socket()?, interface, sock);

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