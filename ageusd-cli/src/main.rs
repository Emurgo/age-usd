#[macro_use]
extern crate json;

mod ascii;
mod fetch_boxes;
mod scan;

use ageusd_headless::bank::BankBox;
use ageusd_headless::input_boxes::{ErgUsdOraclePoolBox, ErgsBox, ReserveCoinBox, StableCoinBox};
use docopt::Docopt;
use ergo_node_interface::local_config::{
    create_new_local_config_file, does_local_config_exist, new_interface_from_local_config,
};
use ergo_node_interface::{NodeInterface, Scan};
use scan::{
    register_all_ballot_tokens_scan, register_bank_scan, register_oracle_pool_scan,
    register_reservecoin_scan, register_stablecoin_scan, register_update_box_scan,
    register_user_ballot_token_scan,
};
use serde::Deserialize;

use ageusd_headless::parameters::*;
use ageusd_headless::protocol::StableCoinProtocol;
use ageusd_headless::update::{BallotBox, UpdateBox};
use ergo_headless_dapp_framework::{nano_erg_to_erg, NanoErg, P2PKAddressString, P2SAddressString};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

const USAGE: &'static str = r#"
Usage:
        stablecoin_cli status
        stablecoin_cli parameters
        stablecoin_cli scans register
        stablecoin_cli scans check
        stablecoin_cli mint ageusd <dollar-amount>
        stablecoin_cli mint reservecoin <amount>
        stablecoin_cli redeem ageusd <dollar-amount>
        stablecoin_cli redeem reservecoin <amount>
        stablecoin_cli vote collect <address>
        stablecoin_cli vote <address>
        stablecoin_cli update <address>
"#;

#[derive(Debug, Deserialize)]
struct Args {
    cmd_status: bool,
    cmd_parameters: bool,
    cmd_scans: bool,
    cmd_register: bool,
    cmd_check: bool,
    cmd_mint: bool,
    cmd_redeem: bool,
    cmd_ageusd: bool,
    cmd_reservecoin: bool,
    cmd_vote: bool,
    cmd_collect: bool,
    cmd_update: bool,
    arg_amount: u64,
    arg_dollar_amount: String,
    arg_address: String,
}

/// A struct which holds all of the StableCoin Protocol `Scan`s
struct ProtocolScans {
    pub bank_scan: Scan,
    pub user_reservecoins_scan: Scan,
    pub user_stablecoins_scan: Scan,
    pub oracle_pool_scan: Scan,
    pub user_update_ballot_scan: Scan,
    pub all_update_ballots_scan: Scan,
    pub update_box_scan: Scan,
}

fn main() {
    print!("{}[2J", 27 as char);
    println!("{}", ascii::ASCII_TITLE);

    // Get the `NodeInterface`
    let node = get_node_interface();

    // Read command line arguments
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // The user's first address in their Ergo Node wallet
    let user_address = node.wallet_addresses().unwrap()[0].clone();

    // Print the current status of the StableCoin Protocol
    if args.cmd_status {
        // Acquiring Scans
        let scans = get_protocol_scans(&node);
        // Create `ErgUsdOraclePoolBox`
        let oracle_box =
            ErgUsdOraclePoolBox::new(&scans.oracle_pool_scan.get_box().unwrap()).unwrap();
        // Create the Bank Box
        let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

        println!("Circulating Status\n===================");
        println!(
            "Amount Of Circulating AgeUSD: ${}",
            (bank_box.num_circulating_stablecoins() as f64 / 100.0)
        );
        println!(
            "Amount Of Circulating ReserveCoins: {}",
            bank_box.num_circulating_reservecoins()
        );

        println!("\nPrices\n=======");
        println!(
            "AgeUSD Nominal Price: {} Ergs",
            nano_erg_to_erg(bank_box.stablecoin_nominal_price(&oracle_box) * 100)
        );
        println!(
            "ReserveCoin Nominal Price: {} Ergs",
            nano_erg_to_erg(bank_box.reservecoin_nominal_price(&oracle_box))
        );

        println!("\nBank Status\n============");
        println!(
            "Current Reserve Ratio: {}%",
            bank_box.current_reserve_ratio(&oracle_box)
        );
        println!(
            "Base Reserves: {} Ergs",
            nano_erg_to_erg(bank_box.base_reserves())
        );
        println!(
            "AgeUSD Liabilities: {} Ergs",
            nano_erg_to_erg(bank_box.liabilities(&oracle_box))
        );
        println!(
            "Equity: {} Ergs",
            nano_erg_to_erg(bank_box.equity(&oracle_box))
        );

        println!("\nAvailable To Mint\n============");
        println!(
            "{} AgeUSD",
            (bank_box.num_able_to_mint_stablecoin(&oracle_box) as f64 / 100.0)
        );
        println!(
            "{} ReserveCoins",
            bank_box
                .num_able_to_mint_reservecoin(&oracle_box, node.current_block_height().unwrap())
        );

        println!("\nUser Wallet Status\n======================");
        println!(
            "Node Wallet Balance: {} Ergs",
            nano_erg_to_erg(node.wallet_nano_ergs_balance().unwrap())
        );
        let rc_boxes = get_reservecoin_boxes(&node);
        let rc_total = ReserveCoinBox::sum_token_amount(&rc_boxes);
        println!("ReserveCoins Owned: {}", rc_total);
        let sc_boxes = get_stablecoin_boxes(&node);
        let sc_total = StableCoinBox::sum_token_amount(&sc_boxes);
        println!("AgeUSD Owned: ${}", sc_total as f64 / 100.0);
    }

    // Print out the protocol parameters
    if args.cmd_parameters {
        println!("Minimum Box Value: {}\nMinimum Reserve Ratio: {}\nMaximum Reserve Ratio: {}\nReserveCoin Default Price: {}\nStableCoin Token ID: {}\nReserveCoin Token ID: {}\nBank NFT ID: {}\nOracle Pool NFT ID: {}\nUpdate Ballot Token ID: {}\nUpdate NFT ID: {}",
    MIN_BOX_VALUE, MIN_RESERVE_RATIO, MAX_RESERVE_RATIO, RESERVECOIN_DEFAULT_PRICE, STABLECOIN_TOKEN_ID, RESERVECOIN_TOKEN_ID, BANK_NFT_ID, ORACLE_POOL_NFT_ID, UPDATE_BALLOT_TOKEN_ID, UPDATE_NFT_ID);
    }

    // Register UTXO-set scans with the provided Ergo Node
    if args.cmd_scans && args.cmd_register {
        let bank_scan = register_bank_scan(&node).unwrap();
        println!("Bank Scan Registered.");
        let user_rc_scan = register_reservecoin_scan(&node).unwrap();
        println!("User ReserveCoins Scan Registered.");
        let user_sc_scan = register_stablecoin_scan(&node).unwrap();
        println!("User StableCoins Scan Registered.");
        let oracle_pool_scan = register_oracle_pool_scan(&node).unwrap();
        println!("Oracle Pool Scan Registered.");
        let user_update_ballot_scan = register_user_ballot_token_scan(&node).unwrap();
        println!("User Update Ballot Scan Registered.");
        let all_update_ballots_scan = register_all_ballot_tokens_scan(&node).unwrap();
        println!("All Update Ballots Scan Registered.");
        let update_box_scan = register_update_box_scan(&node).unwrap();
        println!("Update Box Scan Registered.");

        let scans = vec![
            bank_scan,
            user_rc_scan,
            user_sc_scan,
            oracle_pool_scan,
            user_update_ballot_scan,
            all_update_ballots_scan,
            update_box_scan,
        ];
        ergo_node_interface::Scan::save_scan_ids_locally(scans).unwrap();
        println!("Scan IDs saved locally.");
    }

    // Print out the scan boxes
    if args.cmd_scans && args.cmd_check {
        let scans = get_protocol_scans(&node);
        println!("---------------------------------------------");
        println!("Bank Scan: {:?}", scans.bank_scan.get_boxes().unwrap());
        println!("---------------------------------------------");
        println!(
            "Oracle Pool Scan: {:?}",
            scans.oracle_pool_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
        println!(
            "User StableCoins Scan: {:?}",
            scans.user_stablecoins_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
        println!(
            "User ReserveCoins Scan: {:?}",
            scans.user_reservecoins_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
        println!(
            "User Update Ballot Scan: {:?}",
            scans.user_update_ballot_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
        println!(
            "All Update Ballots Scan: {:?}",
            scans.all_update_ballots_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
        println!(
            "Update Box Scan: {:?}",
            scans.update_box_scan.get_boxes().unwrap()
        );
        println!("---------------------------------------------");
    }

    // Mint StableCoins Action
    if args.cmd_mint && args.cmd_ageusd {
        let us_cent_amount = ((args.arg_dollar_amount.parse::<f64>().unwrap()) * 100.0) as u64;
        mint_stablecoins(us_cent_amount, user_address.clone(), &node);
    }

    // Mint ReserveCoins Action
    if args.cmd_mint && args.cmd_reservecoin {
        mint_reservecoins(args.arg_amount, user_address.clone(), &node);
    }

    // Redeem StableCoins Action
    if args.cmd_redeem && args.cmd_ageusd {
        let us_cent_amount = ((args.arg_dollar_amount.parse::<f64>().unwrap()) * 100.0) as u64;
        redeem_stablecoins(us_cent_amount, user_address.clone(), &node);
    }

    // Redeem ReserveCoins Action
    if args.cmd_redeem && args.cmd_reservecoin {
        redeem_reservecoins(args.arg_amount, user_address.clone(), &node);
    }

    // Collects votes and updates the `Update Box` with the results
    // of the vote.
    if args.cmd_vote && args.cmd_collect {
        collect_votes_for_update(&args.arg_address, &user_address, &node);
    }
    // Issue a vote for updating the protocol
    else if args.cmd_vote {
        vote_for_update(&args.arg_address, &user_address, &node);
    }

    // Issue a vote for updating the protocol
    if args.cmd_update {
        update_protocol(&args.arg_address, &user_address, &node);
    }
}

/// Update Protocol
fn update_protocol(
    update_address: &P2SAddressString,
    user_address: &P2PKAddressString,
    node: &NodeInterface,
) -> String {
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Creating the Update Box
    let update_box = UpdateBox::new(&scans.update_box_scan.get_box().unwrap()).unwrap();
    // Create the Bank Box
    let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

    // Specify the tx fee
    let transaction_fee = 2000000;

    // Get a box for the transaction fee
    let ergs_box_for_fee = ErgsBox::new(&node.highest_value_unspent_box().unwrap()).unwrap();

    let unsigned_tx = protocol
        .action_update_protocol(
            update_address,
            &update_box,
            &bank_box,
            node.current_block_height().unwrap(),
            transaction_fee,
            &ergs_box_for_fee,
            user_address,
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("Update Protocol Tx Id: {}", tx_id);

    tx_id
}

/// Collect Votes for an Update
fn collect_votes_for_update(
    address_voted_for: &P2SAddressString,
    user_address: &P2PKAddressString,
    node: &NodeInterface,
) -> String {
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Creating the Update Box
    let update_box = UpdateBox::new(&scans.update_box_scan.get_box().unwrap()).unwrap();

    // Acquire all of the Ballots
    let all_boxes = scans.all_update_ballots_scan.get_boxes().unwrap();
    // Filter out all non-valid `BallotBox`es
    let mut filtered_ballot_boxes: Vec<BallotBox> = vec![];
    for b in all_boxes {
        if let Ok(bb) = BallotBox::new(&b) {
            filtered_ballot_boxes.push(bb)
        }
    }

    // Get a box for the transaction fee
    let ergs_box_for_fee = ErgsBox::new(&node.highest_value_unspent_box().unwrap()).unwrap();

    // Specify the tx fee
    let transaction_fee = 2000000;

    let unsigned_tx = protocol
        .action_collect_votes(
            address_voted_for,
            &filtered_ballot_boxes,
            &update_box,
            node.current_block_height().unwrap(),
            transaction_fee,
            &ergs_box_for_fee,
            user_address,
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("Collect Update Votes Tx Id: {}", tx_id);

    tx_id
}

/// Vote for an Update
fn vote_for_update(
    address_to_vote_for: &P2SAddressString,
    user_address: &P2PKAddressString,
    node: &NodeInterface,
) -> String {
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Fetch the user's "BallotBox"
    // let user_ballot_box = fetch_user_ballot_box(user_address);
    let user_ballot_box =
        BallotBox::new(&scans.user_update_ballot_scan.get_box().unwrap()).unwrap();

    // Fetch the "UpdateBox"
    // let update_box = fetch_update_box();
    // Creating the Update Box
    let update_box = UpdateBox::new(&scans.update_box_scan.get_box().unwrap()).unwrap();

    // Get a box for the transaction fee
    let ergs_box_for_fee = ErgsBox::new(&node.highest_value_unspent_box().unwrap()).unwrap();

    // Specify the tx fee
    let transaction_fee = 2000000;

    let unsigned_tx = protocol
        .action_vote_for_update(
            address_to_vote_for.clone(),
            user_ballot_box,
            update_box,
            node.current_block_height().unwrap(),
            transaction_fee,
            ergs_box_for_fee,
            user_address.clone(),
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("Vote For Update Tx Id: {}", tx_id);

    tx_id
}

/// Mint ReserveCoins
fn mint_reservecoins(amount: u64, user_address: P2PKAddressString, node: &NodeInterface) -> String {
    println!("Minting ReserveCoins");
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Create `ErgUsdOraclePoolBox`
    let oracle_box = ErgUsdOraclePoolBox::new(&scans.oracle_pool_scan.get_box().unwrap()).unwrap();
    // Create the Bank Box
    let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

    // Specify the tx fee
    let transaction_fee = 2000000;
    // Calculate how many nanoErgs required
    let nano_ergs_required =
        bank_box.total_cost_to_mint_reservecoin(amount, &oracle_box, transaction_fee);
    // Select boxes that cover the minimum required nanoErgs
    let ergs_boxes = get_ergs_boxes_to_cover(nano_ergs_required, &node);

    // Creating the unsigned tx
    let unsigned_tx = protocol
        .action_mint_reservecoin(
            amount,
            user_address,
            transaction_fee,
            node.current_block_height().unwrap(),
            &oracle_box,
            &bank_box,
            &ergs_boxes,
            "9iHyKxXs2ZNLMp9N9gbUT9V8gTbsV7HED1C1VhttMfBUMPDyF7r".to_string(),
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("ReserveCoin Mint Tx Id: {}", tx_id);

    tx_id
}

/// Mint StableCoins
fn mint_stablecoins(amount: u64, user_address: P2PKAddressString, node: &NodeInterface) -> String {
    println!("Minting StableCoins");
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Create `ErgUsdOraclePoolBox`
    let oracle_box = ErgUsdOraclePoolBox::new(&scans.oracle_pool_scan.get_box().unwrap()).unwrap();
    // Create the Bank Box
    let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

    // Specify the tx fee
    let transaction_fee = 2000000;
    // Calculate how many nanoErgs required
    let nano_ergs_required =
        bank_box.total_cost_to_mint_stablecoin(amount, &oracle_box, transaction_fee);
    // Select boxes that cover the minimum required nanoErgs
    let ergs_boxes = get_ergs_boxes_to_cover(nano_ergs_required, &node);

    // Creating the unsigned tx
    let unsigned_tx = protocol
        .action_mint_stablecoin(
            amount,
            user_address,
            transaction_fee,
            node.current_block_height().unwrap(),
            &oracle_box,
            &bank_box,
            &ergs_boxes,
            "9iHyKxXs2ZNLMp9N9gbUT9V8gTbsV7HED1C1VhttMfBUMPDyF7r".to_string(),
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("AgeUSD Mint Tx Id: {}", tx_id);

    tx_id
}

/// Redeem ReserveCoins
fn redeem_reservecoins(
    amount: u64,
    user_address: P2PKAddressString,
    node: &NodeInterface,
) -> String {
    println!("Redeeming ReserveCoins");
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Create `ErgUsdOraclePoolBox`
    let oracle_box = ErgUsdOraclePoolBox::new(&scans.oracle_pool_scan.get_box().unwrap()).unwrap();
    // Create the Bank Box
    let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

    // Creating the unsigned tx
    let unsigned_tx = protocol
        .action_redeem_reservecoin(
            amount,
            user_address,
            2000000,
            node.current_block_height().unwrap(),
            &oracle_box,
            &bank_box,
            &get_reservecoin_boxes(node),
            "9iHyKxXs2ZNLMp9N9gbUT9V8gTbsV7HED1C1VhttMfBUMPDyF7r".to_string(),
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("ReserveCoin Redeem Tx Id: {}", tx_id);

    tx_id
}

/// Redeem StableCoins
fn redeem_stablecoins(
    amount: u64,
    user_address: P2PKAddressString,
    node: &NodeInterface,
) -> String {
    println!("Redeeming StableCoins");
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire protocol
    let protocol = StableCoinProtocol::new();

    // Create `ErgUsdOraclePoolBox`
    let oracle_box = ErgUsdOraclePoolBox::new(&scans.oracle_pool_scan.get_box().unwrap()).unwrap();
    // Create the Bank Box
    let bank_box = BankBox::new(&scans.bank_scan.get_box().unwrap()).unwrap();

    // Creating the unsigned tx
    let unsigned_tx = protocol
        .action_redeem_stablecoin(
            amount,
            user_address,
            2000000,
            node.current_block_height().unwrap(),
            &oracle_box,
            &bank_box,
            &get_stablecoin_boxes(node),
            "9iHyKxXs2ZNLMp9N9gbUT9V8gTbsV7HED1C1VhttMfBUMPDyF7r".to_string(),
        )
        .unwrap();

    let tx_id = node.sign_and_submit_transaction(&unsigned_tx).unwrap();

    println!("AgeUSD Redeem Tx Id: {}", tx_id);

    tx_id
}

/// Small error checking function for acquiring data for a `NodeInterface` /
/// from a local file.
fn get_node_interface() -> NodeInterface {
    // `Node-interface.yaml` setup logic
    if !does_local_config_exist() {
        println!("Could not find local `node-interface.yaml` file.\nCreating said file with basic defaults.\nPlease edit the yaml file and update it with your node parameters to ensure the CLI app can proceed.");
        create_new_local_config_file().ok();
        std::process::exit(0);
    }
    // Error checking reading the local node interface yaml
    if let Err(e) = new_interface_from_local_config() {
        println!(
            "Could not parse local `node-interface.yaml` file.\nError: {:?}",
            e
        );
        std::process::exit(0);
    }
    // Create `NodeInterface`
    new_interface_from_local_config().unwrap()
}

/// Small error checking function for acquiring `Scan`s
/// by using the IDs from a local file.
fn get_protocol_scans(node: &NodeInterface) -> ProtocolScans {
    let res = Scan::read_local_scan_ids(&node);
    if let Err(e) = res {
        println!(
            "An error has occurred while attempting to retrieve the UTXO-set Scans: {:?}",
            e
        );
        std::process::exit(0);
    }
    let all_scans = res.unwrap();

    return ProtocolScans {
        bank_scan: all_scans[0].clone(),
        user_reservecoins_scan: all_scans[1].clone(),
        user_stablecoins_scan: all_scans[2].clone(),
        oracle_pool_scan: all_scans[3].clone(),
        user_update_ballot_scan: all_scans[4].clone(),
        all_update_ballots_scan: all_scans[5].clone(),
        update_box_scan: all_scans[6].clone(),
    };
}

/// Acquire all of the boxes holding ReserveCoins
pub fn get_reservecoin_boxes(node: &NodeInterface) -> Vec<ReserveCoinBox> {
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire ReserveCoin boxes from user's wallet
    let boxes = scans.user_reservecoins_scan.get_boxes().unwrap();
    let reservecoin_boxes = boxes
        .into_iter()
        .map(|b| ReserveCoinBox::new(&b).unwrap())
        .collect();

    reservecoin_boxes
}

/// Acquire all of the boxes holding StableCoins
pub fn get_stablecoin_boxes(node: &NodeInterface) -> Vec<StableCoinBox> {
    // Acquiring Scans
    let scans = get_protocol_scans(&node);
    // Acquire ReserveCoin boxes from user's wallet
    let boxes = scans.user_stablecoins_scan.get_boxes().unwrap();
    let stablecoin_boxes = boxes
        .into_iter()
        .map(|b| StableCoinBox::new(&b).unwrap())
        .collect();

    stablecoin_boxes
}

/// Wrapper function to acquire boxes to cover a given NanoErg amount
/// already wrapped as `ErgsBox`
pub fn get_ergs_boxes_to_cover(amount: NanoErg, node: &NodeInterface) -> Vec<ErgsBox> {
    println!("Ergs required: {}", nano_erg_to_erg(amount));

    let ergo_boxes = node.unspent_boxes_with_min_total(amount).unwrap();
    // Convert selected `ErgoBox`es to `ErgsBox`es
    ergo_boxes
        .into_iter()
        .map(|b| ErgsBox::new(&b).unwrap())
        .collect()
}
