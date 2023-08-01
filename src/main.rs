mod dhcp;
mod send_dhcp;
mod mac;

use std::error::Error;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;
use clap::Parser;
use libc::{sleep, wait};
use pnet::util::Octets;

use local_net::{down, up};
use local_net::get_interface_names;
use local_net::set_route;
use crate::send_dhcp::{get_network, Network};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct InlineArgs {
    /// Network interface
    interface: Option<String>
}

struct Args {
    interface: String
}

fn interactive_cli() -> Args {
    let mut input = String::new();
    let interface_names: Vec<String> = get_interface_names();

    while !interface_names.contains(&input) {
        println!("Choose an interface ({:?})",
                 interface_names);

        io::stdin()
            .read_line(&mut input);

        input = input.trim().to_string();
    }

    Args{ interface: input }
}

fn get_subnet_limits(network: &Network) -> (Ipv4Addr, Ipv4Addr) {
    let netmask_octets = network.get_netmask().unwrap().octets();
    let base_ip_octets = network.get_gateway().unwrap().octets();

    let octect_pairs = netmask_octets.iter().zip(base_ip_octets.iter());

    let mut lower_limit: [u8; 4] = octect_pairs.clone().map(
        |(netmask, base_ip)| {
            base_ip & netmask
        })
        .collect::<Vec<u8>>()
        .try_into().unwrap();
    lower_limit[3] += 1;

    let mut upper_limit: [u8; 4] = octect_pairs.map(
        |(netmask, base_ip)| {
            base_ip | !netmask
        })
        .collect::<Vec<u8>>()
        .try_into().unwrap();
    upper_limit[3] -= 1;

    (Ipv4Addr::from(lower_limit), Ipv4Addr::from(upper_limit))
}

fn cli_get_device_addr (interface: &String) -> Ipv4Addr {
    // Limits:
    let (lower_limit, upper_limit) = get_subnet_limits(&get_network(interface).unwrap());
    let lower_octets = lower_limit.octets();
    let upper_octets = upper_limit.octets();

    // Part of the address that will not change (I.E. 192.168.1):
    let mut base_octets: Vec<u8> = Vec::new();
    let mut base_octets_str: String = String::new();

    println!("Choose an address: (format: constant.constant.lower-upper.lower-upper)");
    for i in 0..4 {
        if lower_octets[i] == upper_octets[i] {
            print!("{}", lower_octets[i]);
            base_octets.push(lower_octets[i]);
            base_octets_str.push_str(&lower_octets[i].to_string());
            base_octets_str.push_str(".");
        } else {
            print!("{}-{}", lower_octets[i], upper_octets[i]);
        }

        if i < 3 {
            print!(".");
        }
    }
    print!("\n");

    for i in 0..3 {
        if lower_octets[i] == upper_octets[i] {
            base_octets.push(lower_octets[i]);
        } else {
            break;
        }
    }

    let validate_input = |s: &String| -> bool {
        let addr = match Ipv4Addr::from_str(&s) {
            Ok(addr) => addr,
            Err(_) => return false
        };

        for i in 0..3 {
            if addr.octets()[i] < lower_octets[i] || addr.octets()[i] > upper_octets[i] {
                return false;
            }
        }
        true
    };

    let mut input = String::new();
    while !validate_input(&input) {
        input = base_octets_str.clone();
        print!("{}", base_octets_str.trim());
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input);

        input = input.trim().to_string();
    }

    println!("You chose {}", input);
    Ipv4Addr::from_str(&*input).unwrap()
}

#[tokio::main]
async fn main() {
    let inline_args = InlineArgs::parse();

    let args = match inline_args.interface {
        Some(interface) => {
            let interfaces = get_interface_names();

            if !interfaces.contains(&interface) {
                println!("{} not found. Use one of the following interfaces: {:?}",
                         interface,
                         interfaces);
                return;
            }

            Args { interface }
        }
        None => {
            interactive_cli()
        }
    };

    let client_addr = cli_get_device_addr(&args.interface);

    let network = get_network(&args.interface).unwrap();
    dbg!(&network);

    down(&args.interface);
    let a = up(&args.interface, client_addr).await.unwrap();

    loop {
        match set_route(&args.interface, network.get_gateway().unwrap()) {
            Ok(_) => break,
            Err(e) => {
                dbg!(&e, &args.interface, network.get_gateway());
            }
        }

        dbg!(&args.interface, &network.get_gateway());
        unsafe { sleep(1); }
    }

}