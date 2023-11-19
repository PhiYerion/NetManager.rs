mod dhcp;
mod mac;
mod send_dhcp;
mod subnet_manager;
mod user_interface;

use pnet::datalink::interfaces;

use local_net::NetworkConfigHandler;

#[tokio::main]
async fn main() {
    let nch = NetworkConfigHandler::new().unwrap();
    let mut iface_idx = 0;
    for interface in interfaces() {
        println!("Interface: {:?}", interface);
        if interface.name == "enp7s0" {
            iface_idx = interface.index
        }
    }
    println!("Interface index: {}", iface_idx);
    for address in nch.get_addresses(iface_idx).await {
        println!("Address: {:?}", address);
    }
}
