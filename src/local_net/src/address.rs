use futures::TryStreamExt;
use netlink_packet_route::address::Nla::Address;
use netlink_packet_route::AddressMessage;
use rtnetlink::Error::RequestFailed;
use rtnetlink::{new_connection, Handle};
use std::net::IpAddr;

pub async fn get_addresses(
    handle: &Handle,
    iface_idx: u32,
) -> Result<Vec<AddressMessage>, Box<dyn std::error::Error>> {
    let mut addresses = handle
        .address()
        .get()
        .set_link_index_filter(iface_idx)
        .execute();
    let mut addresses_vec = Vec::new();

    while let Some(route_message) = addresses.try_next().await? {
        addresses_vec.push(route_message);
    }

    Ok(addresses_vec)
}

pub async fn add_address(
    handle: &Handle,
    iface_idx: u32,
    address: IpAddr,
    prefix_len: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = handle.address().add(iface_idx, address, prefix_len);

    request.execute().await?;

    // Validate:
    for res_addr in get_addresses(handle, iface_idx).await? {
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

pub async fn del_address(
    handle: &Handle,
    address: AddressMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(handle.address().del(address).execute().await?)
}

pub async fn flush_addresses(
    handle: &Handle,
    iface_idx: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    for addr in get_addresses(&handle, iface_idx).await? {
        handle.address().del(addr).execute().await?;
    }

    Ok(())
}
