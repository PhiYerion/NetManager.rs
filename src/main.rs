mod dhcp;
mod mac;
mod send_dhcp;
mod subnet_manager;
mod user_interface;

use clap::Parser;

use crate::send_dhcp::get_network;
use crate::user_interface::{cli_get_device_addr, interactive_cli, Args, InlineArgs};
use local_net::get_interface_names;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    let inline_args = InlineArgs::parse();

    let args = match inline_args.interface {
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
    dbg!(&network);

    //down(&args.interface);
    //up(&args.interface, client_addr).await.unwrap();
}
