use std::net::Ipv4Addr;
use rtnetlink::{new_connection, Error, Handle, IpVersion};
use std::net::IpAddr;
use tokio::runtime::Runtime;
use futures::TryStreamExt;
use netlink_packet_route::RouteMessage;

pub async fn get_routes() -> Result<Vec<RouteMessage>, Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut routes = handle.route().get(IpVersion::V4).execute();
    let mut route_message_vec = Vec::new();

    while let Some(route_message) = routes.try_next().await? {
        route_message_vec.push(route_message);
    };

    Ok(route_message_vec)
}


pub async fn set_default_route(interface: u32, gateway: Ipv4Addr) -> Result<(), Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut request = handle.route().add().v4().replace();

    // Set variables:
    request = request.output_interface(interface);
    request = request.gateway(gateway);

    // Destination for default route:
    request = request.destination_prefix(
        Ipv4Addr::new(0,0,0,0),
        0);

    request.execute().await?;

    Ok(())
}

pub async fn flush_route(address_family: u8, interface_index: u32) -> Result<(), Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut routes = handle.route().get(IpVersion::V4).execute();

    while let Some(route_message) = routes.try_next().await? {
        let handle = handle.clone();
        handle.route().del(route_message).execute().await?;
    };

    Ok(())
}