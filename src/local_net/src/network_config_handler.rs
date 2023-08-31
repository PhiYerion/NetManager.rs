use std::io;
use std::net::{IpAddr, Ipv4Addr};
use netlink_packet_route::{RtnlMessage};
use netlink_proto::Connection;
use rtnetlink::{Handle, new_connection};
use std::io::Error;
use crate::{add_address, set_default_route, flush_addresses};

pub struct NetworkConfigHandler {
    conn: Connection<RtnlMessage>,
    handler: Handle
}

impl NetworkConfigHandler {
    pub fn new() -> Result<NetworkConfigHandler, Error> {
        let (conn, handler, _) = match new_connection() {
            Ok(connection) => connection,
            Err(_) => return Err(Error::from(io::Error::new(io::ErrorKind::Other, "Netlink error")))
        };

        Ok(NetworkConfigHandler {
            conn,
            handler,
        })
    }

    fn reset_connection(&mut self) -> Result<(), std::io::Error> {
        (self.conn, self.handler, _) = new_connection()?;

        Ok(())
    }

    fn handle_rt_error(&mut self) -> Result<(), std::io::Error> {
        self.reset_connection()?;

        // later have configs in this class (or something else)
        // and then flush entire network and redo the sections or
        // all of the network kernel config depending on priority
        // of the request
        Ok(())
    }

    pub async fn flush_addresses(&self, interface: u32) {
        let response = flush_addresses(&self.handler);
    }

    pub async fn add_address(&self, interface: u32, address: IpAddr, prefix_len: u8) -> Result<(), std::io::Error> {
        let response = add_address(&self.handler, interface, address, prefix_len).await;
        match response {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Netlink error"))
        }
    }

    pub async fn set_default_route(&self, interface: u32,  gateway: Ipv4Addr) -> Result<(), std::io::Error> {
        let response = set_default_route(&self.handler, interface, gateway).await;
        match response {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Netlink error"))
        }
    }

}
