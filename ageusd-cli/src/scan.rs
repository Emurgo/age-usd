use crate::Result;
use ergo_node_interface::{NodeInterface, Scan};
use ageusd_headless::parameters::*;

/// Register the scan to find the `Bank` box via the provided `NodeInterface`
pub fn register_bank_scan(node: &NodeInterface) -> Result<Scan> {
    let tracking_rule = object! {
            "predicate": "containsAsset",
            "assetId": BANK_NFT_ID,
    };

    Ok(Scan::register(
        &"StableCoin Bank Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find the StableCoins that the user owns in the very first
/// address of their Ergo node wallet via the provided `NodeInterface`
pub fn register_stablecoin_scan(node: &NodeInterface) -> Result<Scan> {
    let wallet_address = node.wallet_addresses()?[0].clone();
    let user_address_bytes = Scan::serialize_p2pk_for_tracking(&node, &wallet_address)?;

    let tracking_rule = object! {
        "predicate": "and",
        "args": [
            {
            "predicate": "containsAsset",
            "assetId": STABLECOIN_TOKEN_ID.clone(),
            },
            {
            "predicate": "equals",
            "value": user_address_bytes.clone(),
            }
        ]
    };

    Ok(Scan::register(
        &"User Wallet StableCoins Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find the ReserveCoins that the user owns in the very first
/// address of their Ergo node wallet via the provided `NodeInterface`
pub fn register_reservecoin_scan(node: &NodeInterface) -> Result<Scan> {
    let wallet_address = node.wallet_addresses()?[0].clone();
    let user_address_bytes = Scan::serialize_p2pk_for_tracking(&node, &wallet_address)?;

    let tracking_rule = object! {
        "predicate": "and",
        "args": [
            {
            "predicate": "containsAsset",
            "assetId": RESERVECOIN_TOKEN_ID.clone(),
            },
            {
            "predicate": "equals",
            "value": user_address_bytes.clone(),
            }
        ]
    };

    Ok(Scan::register(
        &"User Wallet ReserveCoins Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find the Oracle Pool Box
pub fn register_oracle_pool_scan(node: &NodeInterface) -> Result<Scan> {
    let tracking_rule = object! {
            "predicate": "containsAsset",
            "assetId": ORACLE_POOL_NFT_ID,
    };

    Ok(Scan::register(
        &"Oracle Pool Box Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find the user's vote token. (Must be held by the first address
/// in the Ergo Node wallet)
pub fn register_user_ballot_token_scan(node: &NodeInterface) -> Result<Scan> {
    let wallet_address = node.wallet_addresses()?[0].clone();
    let user_address_bytes = Scan::serialize_p2pk_for_tracking(&node, &wallet_address)?;

    let tracking_rule = object! {
        "predicate": "and",
        "args": [
            {
            "predicate": "containsAsset",
            "assetId": UPDATE_BALLOT_TOKEN_ID.clone(),
            },
            {
            "predicate": "equals",
            "value": user_address_bytes.clone(),
            }
        ]
    };

    Ok(Scan::register(
        &"StableCoin - User Ballot Token Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find all boxes that hold ballot tokens
pub fn register_all_ballot_tokens_scan(node: &NodeInterface) -> Result<Scan> {
    let tracking_rule = object! {
            "predicate": "containsAsset",
            "assetId": UPDATE_BALLOT_TOKEN_ID,
    };

    Ok(Scan::register(
        &"StableCoin - All Ballot Token Boxes Scan".to_string(),
        tracking_rule,
        node,
    )?)
}

/// Register to find the Update box
pub fn register_update_box_scan(node: &NodeInterface) -> Result<Scan> {
    let tracking_rule = object! {
            "predicate": "containsAsset",
            "assetId": UPDATE_NFT_ID,
    };

    Ok(Scan::register(
        &"StableCoin - Update Box Scan".to_string(),
        tracking_rule,
        node,
    )?)
}
