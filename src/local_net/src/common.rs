use default_net::interface::get_interfaces;
use default_net;
use default_net::Interface;

pub fn get_network_interfaces() -> Vec<Interface> {
    get_interfaces()
}

pub fn get_interface_names() -> Vec<String> {
    get_interfaces()
        .iter()
        .map(|interface| interface.name.clone())
        .collect()
}
