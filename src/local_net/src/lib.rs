mod set_net;
mod get_net_iface;
mod common;
mod routes;


pub use crate::common::get_interface_names;
pub use crate::set_net::{up, down};
pub use crate::routes::flush_routes;
pub use crate::routes::get_routes;
pub use crate::routes::set_default_route;