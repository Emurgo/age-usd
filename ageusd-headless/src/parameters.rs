// This file holds hard-coded parameters of the protocol.
use ergo_headless_dapp_framework::NanoErg;

/// The minimum value a box will hold. (aka. Min to cover storage rent)
pub static MIN_BOX_VALUE: NanoErg = 10000000;

// Default price of a ReserveCoin if 0 ReserveCoins are in circulation.
// Primarily set for edgecase to be covered.
pub static RESERVECOIN_DEFAULT_PRICE: NanoErg = 1000000;

// Reserve Ratios
pub static MIN_RESERVE_RATIO: u64 = 400;
pub static MAX_RESERVE_RATIO: u64 = 800;

// The Block Height that the "Bootstrap Cool-Off" period completes & the
// Maximum Reserve Ratio is officially applied thenceforth
pub static COOLING_OFF_HEIGHT: u64 = 377770;

// The fee percentage that is charge on each minting/redeeming action as a
// u64. 1 == 1%
pub static FEE_PERCENT: u64 = 2;
// The fee percentage that users pay to the frontend implementor as a f64.
// 1 == 100%, 0.01 == 1%
pub static IMPLEMENTOR_FEE_PERCENT: f64 = 0.0025;

// Token IDs
pub static STABLECOIN_TOKEN_ID: &str = "";
pub static RESERVECOIN_TOKEN_ID: &str = "";
pub static BANK_NFT_ID: &str = "";
pub static ORACLE_POOL_NFT_ID: &str =
    "008a94c8c76bbaa1f0a346697d1794eb31d94b37e5533af9cc0b6932bf159339";
pub static UPDATE_NFT_ID: &str = "";
pub static UPDATE_BALLOT_TOKEN_ID: &str = "";
