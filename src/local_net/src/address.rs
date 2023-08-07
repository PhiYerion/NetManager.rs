use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
use rtnetlink::{new_connection, IpVersion};
use futures::TryStreamExt;
use netlink_packet_route::{AddressMessage, RouteMessage};
use rtnetlink::Error::RequestFailed;

pub async fn get_addresses() -> Result<Vec<AddressMessage>, Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut addresses = handle.address().get().execute();
    let mut addresses_vec = Vec::new();

    while let Some(route_message) = addresses.try_next().await? {
        addresses_vec.push(route_message);
    };

    Ok(addresses_vec)
}

pub async fn add_address(interface: u32, address: IpAddr, prefix_len: u8) -> Result<(), Box<dyn Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let request = handle.address().add(interface, address, prefix_len);

    match request.execute().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e))
    }
}