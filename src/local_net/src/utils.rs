use futures::TryStreamExt;
use rtnetlink::{new_connection, Handle};

/// Flush all addresses from an interface.
pub async fn flush_addresses(handle: &Handle, iface_idx: u32) -> Result<(), crate::RTNetlinkError> {
    let mut request = handle
        .address()
        .get()
        .set_link_index_filter(iface_idx)
        .execute();

    #[cfg(not(experimental = "threading"))]
    {
        let (connection, delete_handle, _) = new_connection().unwrap();
        tokio::spawn(connection);
        while let Some(address) = request
            .try_next()
            .await
            .map_err(crate::RTNetlinkError::RTNetlink)?
        {
            crate::del_address(&delete_handle, address).await?
        }
    }

    #[cfg(experimental = "threading")]
    {
        use futures::future::join_all;

        let futures = Vec::new();
        while let Some(address) = request
            .try_next()
            .await
            .map_err(RTNetlinkError::RTNetlink)?
        {
            let (connection, delete_handle, _) = new_connection().unwrap();
            del_address(&delete_handle, address);
            futures.push(tokio::spawn(connection));
        }
        join_all(futures).await?;
    }

    // Validation
    let res_address = handle
        .address()
        .get()
        .set_link_index_filter(iface_idx)
        .execute()
        .try_next()
        .await
        .map_err(crate::RTNetlinkError::RTNetlink)?;

    match res_address {
        Some(_) => Err(crate::RTNetlinkError::ValidationFailed),
        None => Ok(()),
    }
}
