use futures::TryStreamExt;
use netlink_packet_route::RouteMessage;
use rtnetlink::Error::RequestFailed;
use rtnetlink::{new_connection, Handle, IpVersion};
use std::net::Ipv4Addr;

pub async fn get_routes(handle: &Handle) -> Result<Vec<RouteMessage>, Box<dyn std::error::Error>> {
    let mut routes = handle.route().get(IpVersion::V4).execute();
    let mut route_message_vec = Vec::new();

    while let Some(route_message) = routes.try_next().await? {
        route_message_vec.push(route_message);
    }

    Ok(route_message_vec)
}

pub async fn set_default_route(
    handle: &Handle,
    interface: u32,
    gateway: Ipv4Addr,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = handle
        // Base route request:
        .route()
        .add()
        .v4()
        .replace()
        // Gateway and interface:
        .output_interface(interface)
        .gateway(gateway)
        // Kernel address:
        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0);

    request.execute().await?;

    Ok(())
}

pub async fn flush_routes(handle: &Handle) -> Result<(), Box<dyn std::error::Error>> {
    let mut routes = handle.route().get(IpVersion::V4).execute();

    while let Some(route_message) = routes.try_next().await? {
        let handle = handle.clone();
        handle.route().del(route_message).execute().await?;
    }

    // Verify and return:
    match get_routes(handle).await?.len() {
        0 => Ok(()),
        _ => Err(Box::new(RequestFailed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlink_packet_route::route::Nla::{Gateway, Oif};

    #[tokio::test]
    async fn no_set_default_route_runtime_error() {
        let gateway = Ipv4Addr::new(192, 168, 1, 1);
        let (connection, handle, _) = new_connection().unwrap();

        flush_routes(&handle).await.unwrap();
        assert!(get_routes(&handle).await.unwrap().len() == 0);

        // There should be a '2' interface
        set_default_route(&handle, 2, gateway).await.unwrap();

        let mut found_route = false;
        for route in get_routes(&handle).await.unwrap() {
            if route.header.table == 254
                && route.header.address_family == 2
                && route.nlas.contains(&Gateway(gateway.octets().to_vec()))
                && route.nlas.contains(&Oif(2))
            {
                found_route = true;
                break;
            }
        }

        assert!(found_route);
    }
}

