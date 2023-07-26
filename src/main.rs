mod dhcp;
mod send_dhcp;
mod get_net_iface;
mod set_net;

use send_dhcp::get_netmask;

fn main() {
    println!("{:?}", get_netmask("enp7s0").unwrap());
}