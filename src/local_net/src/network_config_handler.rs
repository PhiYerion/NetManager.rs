use crate::{add_address, set_default_route};
use rtnetlink::{new_connection, Handle};
use std::io;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr};

pub struct NetworkConfigHandler {
    // This is required to keep the connection alive
    #[allow(dead_code)]
    handler: Handle,
}

impl NetworkConfigHandler {
    pub fn new() -> Result<NetworkConfigHandler, Error> {
        let handler = match new_connection() {
            Ok((conn, handler, _)) => {
                tokio::spawn(conn);
                handler
            }
            Err(_) => {
                return Err(Error::from(io::Error::new(
                    io::ErrorKind::Other,
                    "Netlink error",
                )))
            }
        };

        Ok(NetworkConfigHandler { handler })
    }

    fn start_connection(&mut self) -> Result<(), std::io::Error> {
        let (conn, handler, _) = new_connection()?;
        tokio::spawn(conn);
        self.handler = handler;

        Ok(())
    }

    fn handle_rt_error(&mut self) -> Result<(), std::io::Error> {
        self.start_connection()?;

        // later have configs in this class (or something else)
        // and then flush entire network and redo the sections or
        // all of the network kernel config depending on priority
        // of the request
        Ok(())
    }

    pub async fn flush_addresses(&self, iface_idx: u32) {
        let _ = crate::flush_addresses(&self.handler, iface_idx).await;
    }

    pub async fn get_addresses(&self, iface_idx: u32) -> Vec<netlink_packet_route::AddressMessage> {
        crate::get_addresses(&self.handler, iface_idx)
            .await
            .unwrap()
    }

    pub async fn add_address(
        &self,
        iface_idx: u32,
        address: IpAddr,
        prefix_len: u8,
    ) -> Result<(), std::io::Error> {
        let response = add_address(&self.handler, iface_idx, address, prefix_len).await;
        match response {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Netlink error",
            )),
        }
    }

    pub async fn set_default_route(
        &self,
        interface: u32,
        gateway: Ipv4Addr,
    ) -> Result<(), std::io::Error> {
        let response = set_default_route(&self.handler, interface, gateway).await;
        match response {
            Ok(_) => Ok(()),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Netlink error",
            )),
        }
    }
}
