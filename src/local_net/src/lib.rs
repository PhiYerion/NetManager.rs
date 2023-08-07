mod set_net;
mod set_default_route;
mod get_net_iface;
mod common;
mod kernel_req;


pub use crate::common::get_interface_names;
pub use crate::set_net::{up, down};
pub use crate::set_default_route::set_route;
pub use crate::kernel_req::flush_route;
pub use crate::kernel_req::get_routes;
pub use crate::kernel_req::get_routes_rt;