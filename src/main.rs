mod dhcp;
mod send_dhcp;
mod mac;

use std::io;
use std::net::Ipv4Addr;
use clap::Parser;
use libc::wait;
use tokio::runtime::Runtime;

use send_dhcp::get_netmask;
use crate::get_net_iface::get_network_interfaces;
use crate::set_net::{set_route, up};

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

#[tokio::main]
async fn main() {
    let inline_args = InlineArgs::parse();

    let args = match inline_args.interface {
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

    // println!("{:?}", get_netmask(&args.interface));

    let a = up(&args.interface, Ipv4Addr::new(192, 168, 4, 222)).await.unwrap();
    println!("{}", a);
    let b = set_route().unwrap();

}