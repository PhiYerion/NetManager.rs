mod dhcp;
mod mac;
mod send_dhcp;
mod subnet_manager;
mod user_interface;

use clap::Parser;
use pnet::datalink::interfaces;

use crate::send_dhcp::get_network;
use crate::user_interface::{cli_get_device_addr, interactive_cli, Args, InlineArgs};
use local_net::{flush_addresses, get_addresses, get_interface_names, NetworkConfigHandler};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    let inline_args = InlineArgs::parse();
    let nch = NetworkConfigHandler::new().unwrap();
    let mut iface_idx = 0;
    for interface in interfaces() {
        println!("Interface: {:?}", interface);
        if interface.name == "enp7s0" {
            iface_idx = interface.index
        }
    }
    println!("Interface index: {}", iface_idx);
    for address in nch.get_addresses(iface_idx).await.unwrap() {
        println!("Address: {:?}", address);
    }

    /* let args = match inline_args.interface {
        Some(interface) => {
            let interfaces = get_interface_names();

            if !interfaces.contains(&interface) {
                println!(
                    "{} not found. Use one of the following interfaces: {:?}",
                    interface, interfaces
                );
                return;
            }

            Args { interface }
        }
        None => interactive_cli(),
    };

    let client_addr = cli_get_device_addr(&args.interface);


    let network = get_network(&args.interface).unwrap();
    dbg!(&network); */

    //down(&args.interface);
    //up(&args.interface, client_addr).await.unwrap();
}
