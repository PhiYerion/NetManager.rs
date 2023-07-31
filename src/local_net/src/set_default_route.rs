use std::ffi::CString;
use std::io;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::fd::AsRawFd;
use libc::{AF_INET, sa_family_t, sockaddr};

pub fn gen_sockaddr(addr: Ipv4Addr) -> sockaddr {
    sockaddr {
        sa_family: AF_INET as sa_family_t,
        sa_data: [0, 0,
            addr.octets()[0] as i8,
            addr.octets()[1] as i8,
            addr.octets()[2] as i8,
            addr.octets()[3] as i8,
            0, 0, 0, 0, 0, 0, 0, 0]
    }
}

pub fn get_mut_c_char(input: &str) -> *mut libc::c_char {
    let rt_dev_cstring = CString::new(input).expect("CString::new failed");

    rt_dev_cstring.as_ptr() as *mut libc::c_char
}

pub fn set_route(interface: &str, addr: Ipv4Addr) -> Result<(), io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let rt_gateway = gen_sockaddr(addr);
    let rt_dst = gen_sockaddr(Ipv4Addr::new(0,0,0,0));
    let rt_genmask = gen_sockaddr(Ipv4Addr::new(0,0,0,0));
    let rt_dev = get_mut_c_char(interface);


    let mut rt = libc::rtentry {
        rt_pad1: 0,
        rt_dst,
        rt_gateway,
        rt_genmask,
        rt_flags: libc::RTF_UP | libc::RTF_GATEWAY,
        rt_pad2: 0,
        rt_pad3: 0,
        rt_tos: 0,
        rt_class: 0,
        rt_pad4: [0; 3],
        rt_metric: 0,
        rt_dev,
        rt_mtu: 0,
        rt_window: 0,
        rt_irtt: 0,
    };

    let result = unsafe {
        libc::ioctl(
            socket.as_raw_fd(),
            libc::SIOCADDRT,
            &mut rt as *mut _ as *mut libc::c_void,
        )};


    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())

}