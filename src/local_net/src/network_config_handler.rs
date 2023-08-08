use std::io;
use std::net::IpAddr;
use netlink_packet_route::{AddressMessage, RtnlMessage};
use netlink_proto::Connection;
use rtnetlink::{Error, Handle, new_connection, new_connection_with_socket};
use crate::add_address;

pub struct NetworkConfigHandler {
    conn: Connection<RtnlMessage>,
    handler: Handle
}

impl NetworkConfigHandler {
    pub fn new() -> NetworkConfigHandler {
        let (conn, handler, _) = new_connection().unwrap();

        NetworkConfigHandler {
            conn,
            handler,
        }
    }

    fn reset_connection(&mut self) -> Result<(), std::io::Error> {
        (self.conn, self.handler, _) = new_connection()?;

        Ok(())
    }

    fn handle_rt_error(&mut self, callback: fn()) -> Result<(), std::io::Error> {
        self.reset_connection()?;

        // later have configs in this class (or something else)
        // and then flush entire network and redo the sections or
        // all of the network kernel config depending on priority
        // of the request
        Ok(())
    }

    pub async fn add_address(&self, interface: u32, address: IpAddr, prefix_len: u8) -> Result<(), Box<dyn std::error::Error>> {
        add_address(&self.handler, interface, address, prefix_len).await
    }
}