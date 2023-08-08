use std::net::{IpAddr};
use rtnetlink::{new_connection};
use futures::TryStreamExt;
use netlink_packet_route::{AddressMessage};
use netlink_packet_route::address::Nla::Address;
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

pub async fn add_address(interface: u32, address: IpAddr, prefix_len: u8) -> Result<(), Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let request = handle.address().add(interface, address, prefix_len);

    request.execute().await?;

    // Validate:
    for res_addr in get_addresses().await? {

        match address {
            IpAddr::V4(v4) => {
                if res_addr.nlas.contains(&Address(v4.octets().to_vec())) {
                    return Ok(());
                }
            }
            IpAddr::V6(v6) => {
                if res_addr.nlas.contains(&Address(v6.octets().to_vec())) {
                    return Ok(());
                }
            }
        }

    }

    Err(Box::new(RequestFailed))
}