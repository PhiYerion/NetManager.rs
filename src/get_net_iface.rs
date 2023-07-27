use std::collections::HashSet;
use std::ffi::CStr;
use libc::{self, c_char};
use std::str;

pub fn get_network_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    let mut seen_items: HashSet<String> = HashSet::new();

    let mut ifc: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifc) } == 0 {
        let mut current = ifc;
        while !current.is_null() {
            if let Some(interface_name) = get_interface_name(current) {
                if interface_name != "lo" && !seen_items.contains(&*interface_name) {
                    seen_items.insert(interface_name.clone());
                    interfaces.push(interface_name.clone());
                }
            }
            current = unsafe { (*current).ifa_next };
        }
        unsafe { libc::freeifaddrs(ifc) };
    }

    interfaces
}


fn get_interface_name(ifa: *const libc::ifaddrs) -> Option<String> {
    if ifa.is_null() {
        return None;
    }

    let cstr = unsafe { (*ifa).ifa_name };
    let bytes = unsafe { CStr::from_ptr(cstr).to_bytes() };

    // Convert the bytes to a valid UTF-8 string if possible
    match str::from_utf8(bytes) {
        Ok(interface_name) => Some(interface_name.to_string()),
        Err(_) => None,
    }
}
