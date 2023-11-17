use crate::send_dhcp::{get_network, Network};
use clap::Parser;
use local_net::get_interface_names;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct InlineArgs {
    /// Network interface
    pub interface: Option<String>,
}

pub struct Args {
    pub interface: String,
}

pub fn interactive_cli() -> Args {
    let mut input = String::new();
    let interface_names: Vec<String> = get_interface_names();

    while !interface_names.contains(&input) {
        println!("Choose an interface ({:?})", interface_names);

        io::stdin().read_line(&mut input);

        input = input.trim().to_string();
    }

    Args { interface: input }
}

pub fn cli_get_device_addr(interface: &String) -> Ipv4Addr {
    // Limits:
    let (lower_limit, upper_limit) = get_subnet_limits(&get_network(interface).unwrap());
    let lower_octets = lower_limit.octets();
    let upper_octets = upper_limit.octets();

    // Part of the address that will not change (I.E. 192.168.1):
    let mut base_octets_str: String = String::new();

    // Populate base_octets (can be integrated into above, but less readable)
    for i in 0..3 {
        if lower_octets[i] == upper_octets[i] {
            base_octets_str.push_str(&lower_octets[i].to_string());
            base_octets_str.push_str(".");
        } else {
            break;
        }
    }

    // Print the range
    {
        println!("Choose an address: (format: constant.constant.lower-upper.lower-upper)");

        print!("Lower limit: ");
        for i in 0..4 {
            print!("{}", lower_octets[i]);

            if i < 3 {
                print!(".");
            }
        }

        print!("\nUpper Limit: ");
        for i in 0..4 {
            print!("{}", upper_octets[i]);

            if i < 3 {
                print!(".");
            }
        }
        println!();
    }

    // Get user input
    let get_user_input = || -> Ipv4Addr {
        let mut input = String::new();

        let validate_input = |s: &String| -> Result<Ipv4Addr, io::Error> {
            let addr = match Ipv4Addr::from_str(&s) {
                Ok(addr) => addr,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
            };
            for i in 0..3 {
                if addr.octets()[i] < lower_octets[i] || addr.octets()[i] > upper_octets[i] {
                    return Err(io::Error::new(
                        io::ErrorKind::AddrNotAvailable,
                        "Out of range.",
                    ));
                }
            }
            Ok(addr)
        };

        loop {
            match validate_input(&input) {
                Ok(addr) => return addr,
                Err(_) => {
                    input = base_octets_str.clone();
                    print!("{}", base_octets_str.trim());
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input);

                    input = input.trim().to_string();
                }
            }
        }
    };

    get_user_input()
}

fn get_subnet_limits(network: &Network) -> (Ipv4Addr, Ipv4Addr) {
    let netmask_octets = network.get_netmask().unwrap().octets();
    let base_ip_octets = network.get_gateway().unwrap().octets();

    let octect_pairs = netmask_octets.iter().zip(base_ip_octets.iter());

    let mut lower_limit: [u8; 4] = octect_pairs
        .clone()
        .map(|(netmask, base_ip)| base_ip & netmask)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    lower_limit[3] += 1;

    let mut upper_limit: [u8; 4] = octect_pairs
        .map(|(netmask, base_ip)| base_ip | !netmask)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    upper_limit[3] -= 1;

    (Ipv4Addr::from(lower_limit), Ipv4Addr::from(upper_limit))
}
