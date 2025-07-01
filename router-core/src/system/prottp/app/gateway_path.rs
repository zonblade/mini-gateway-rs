use crate::config::{self, GatewayPath};

// path stay as it is
pub fn init(payload: String) -> Result<(), serde_json::Error> {
    let checksum = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    eprintln!("[-TC-]   Gateway id : {}", checksum);

    let gateway_data = serde_json::from_str::<Vec<GatewayPath>>(&payload);
    let gateway_data = match gateway_data {
        Ok(data) => {
            log::warn!("Parsed gateway data: {:?}", data);
            data
        }
        Err(e) => {
            log::error!("Failed to parse gateway data: {}", e);
            return Err(e);
        }
    };
    log::debug!("Parsed gateway data: {:#?}", gateway_data);

    let gateway_existing = match config::RoutingData::GatewayRouting.xget::<Vec<GatewayPath>>() {
        Some(data) => data,
        None => {
            vec![]
        }
    };

    eprintln!(
        "[-TC-]   Count of existing gateway addresses: {}",
        gateway_existing.len()
    );

    // Get existing gateway addresses
    let gateway_existing: Vec<String> = gateway_existing
        .iter()
        .map(|x| x.addr_bind.clone())
        .collect();

    // Get incoming gateway addresses
    let gateway_incoming: Vec<String> = gateway_data.iter().map(|x| x.addr_bind.clone()).collect();

    // Find addresses that are in existing but not in incoming (to be removed)
    let addresses_to_remove: Vec<String> = gateway_existing
        .clone()
        .into_iter()
        .filter(|x| !gateway_incoming.contains(x))
        .collect();

    // Find addresses that are in incoming but not in existing (to be added)
    let addresses_to_add: Vec<String> = gateway_incoming
        .iter()
        .filter(|x| !gateway_existing.contains(x))
        .cloned()
        .collect();

    eprintln!(
        "[-TC-]   Addresses to remove: {:?}",
        addresses_to_remove.len()
    );
    eprintln!("[-TC-]   Addresses to add: {:?}", addresses_to_add.len());

    config::RoutingData::GatewayID.set(&checksum);
    config::RoutingData::GatewayRouting.xset(&gateway_data);

    Ok(())
}
