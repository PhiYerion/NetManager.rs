mod dhcp;
mod send_dhcp;
mod mac;
mod user_interface;

use std::error::Error;
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;
use clap::Parser;
use libc::AF_INET;
use pnet::util::Octets;

use local_net::{down, flush_routes, get_routes, set_default_route, up};
use local_net::get_interface_names;
use crate::send_dhcp::{get_network};
use crate::user_interface::{Args, cli_get_device_addr, InlineArgs, interactive_cli};

#[tokio::main]
async fn main() {
    // let inline_args = InlineArgs::parse();
    //
    // let args = match inline_args.interface {
    //     Some(interface) => {
    //         let interfaces = get_interface_names();
    //
    //         if !interfaces.contains(&interface) {
    //             println!("{} not found. Use one of the following interfaces: {:?}",
    //                      interface,
    //                      interfaces);
    //             return;
    //         }
    //
    //         Args { interface }
    //     }
    //     None => {
    //         interactive_cli()
    //     }
    // };
    //
    // let client_addr = cli_get_device_addr(&args.interface);
    //
    // let network = get_network(&args.interface).unwrap();
    // dbg!(&network);
    //
    // down(&args.interface);
    // up(&args.interface, client_addr).await.unwrap();
    //
    // loop {
    //     match set_route(&args.interface, network.get_gateway().unwrap()) {
    //         Ok(_) => break,
    //         Err(e) => {
    //             dbg!(&e, &args.interface, network.get_gateway());
    //         }
    //     }
    //
    //     dbg!(&args.interface, &network.get_gateway());
    //     std::thread::sleep(Duration::from_secs(1));
    // }
    let a = get_routes().await;
    print!("{:?}", a);
}