use libc::{self};
use default_net;
use default_net::Interface;

pub fn get_network_interfaces() -> Vec<Interface> {
    default_net::interface::get_interfaces()
}


// fn get_interface_name(ifa: *const libc::ifaddrs) -> Option<String> {
//     if ifa.is_null() {
//         return None;
//     }
//
//     let cstr = unsafe { (*ifa).ifa_name };
//     let bytes = unsafe { CStr::from_ptr(cstr).to_bytes() };
//
//     // Convert the bytes to a valid UTF-8 string if possible
//     match str::from_utf8(bytes) {
//         Ok(interface_name) => Some(interface_name.to_string()),
//         Err(_) => None,
//     }
// }
