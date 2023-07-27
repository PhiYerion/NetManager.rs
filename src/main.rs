mod dhcp;
mod send_dhcp;
mod get_net_iface;
mod set_net;

use std::collections::{HashSet};
use std::io;
use clap::Parser;
use pnet::util::MacAddr;

use send_dhcp::get_netmask;
use crate::get_net_iface::get_network_interfaces;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct inlineArgs {
    /// Network interface
    interface: Option<String>
}

struct Args {
    interface: String
}

fn interactive_cli() -> Args {
    let mut input = String::new();
    let interfaces = get_network_interfaces();

    while !interfaces.contains(&input) {
        println!("Choose an interface ({:?})",
                 get_network_interfaces());

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        input = input.trim().to_string();
    }

    Args{ interface: input }
}

fn main() {
    let inlineArgs = inlineArgs::parse();

    let args = match inlineArgs.interface {
        Some(interface) => {
            let interfaces = get_network_interfaces();

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

    println!("{:?}", get_netmask(&args.interface, MacAddr(0x18, 0xc0, 0x4d, 0x5b, 0x03, 0xae)).unwrap());
}