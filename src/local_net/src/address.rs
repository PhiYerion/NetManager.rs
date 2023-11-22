use futures::TryStreamExt;
use netlink_packet_route::AddressMessage;
use rtnetlink::Handle;
use std::net::IpAddr;

#[derive(Debug)]
pub enum RTNetlinkError {
    RTNetlink(rtnetlink::Error),
    IOError(std::io::Error),
    ValidationFailed,
}

pub async fn add_address(
    handle: &Handle,
    iface_idx: u32,
    address: IpAddr,
    prefix_len: u8,
) -> Result<(), RTNetlinkError> {
    let request = handle.address().add(iface_idx, address, prefix_len);

    request.execute().await.map_err(RTNetlinkError::RTNetlink)?;

    // Validate:
    let res_address = handle
        .address()
        .get()
        .set_link_index_filter(iface_idx)
        .set_address_filter(address)
        .execute()
        .try_next()
        .await
        .map_err(RTNetlinkError::RTNetlink)?;

    match res_address {
        Some(_) => Ok(()),
        None => Err(RTNetlinkError::ValidationFailed),
    }
}

pub async fn del_address(handle: &Handle, address: AddressMessage) -> Result<(), RTNetlinkError> {
    let request = handle.address().del(address);
    request.execute().await.map_err(RTNetlinkError::RTNetlink)?;

    // Validation cannot be done at this moment because nlas feilds are private. We will have to
    // manually modify this package to to validation.
    Ok(())
}
