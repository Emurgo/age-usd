use ageusd_headless::update::{BallotBox, UpdateBox};
use ergo_headless_dapp_framework::{ErgUsdOraclePoolBox, ExplorerFindable, SpecifiedBox};
use reqwest::blocking::get;

/// Fetch the user's `BallotBox` from the public Ergo Explorer API
pub fn fetch_user_ballot_box(user_address: &str) -> BallotBox {
    let box_spec = BallotBox::box_spec().modified_address(Some(user_address.to_string()));
    let ballot_box_url = box_spec
        .explorer_endpoint("https://api.ergoplatform.com/api")
        .unwrap();
    println!("URL: {}", ballot_box_url);
    let response = get(&ballot_box_url).unwrap().text().unwrap();
    let bbs = BallotBox::process_explorer_response(&response).unwrap();
    if bbs.len() == 0 {
        println!("Response Bad: {}", response);
    }
    bbs[0].clone()
}

/// Fetch all `BallotBox`es from the public Ergo Explorer API
pub fn fetch_all_ballot_boxes() -> Vec<BallotBox> {
    let ballot_box_url = BallotBox::box_spec()
        .explorer_endpoint("https://api.ergoplatform.com/api")
        .unwrap();
    let response = get(&ballot_box_url).unwrap().text().unwrap();
    BallotBox::process_explorer_response(&response).unwrap()
}

/// Fetch the `UpdateBox` from the public Ergo Explorer API
pub fn fetch_update_box() -> UpdateBox {
    let url = UpdateBox::box_spec()
        .explorer_endpoint("https://api.ergoplatform.com/api")
        .unwrap();
    let response = get(&url).unwrap().text().unwrap();
    UpdateBox::process_explorer_response(&response).unwrap()[0].clone()
}
