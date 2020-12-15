use ergo_headless_dapp_framework::HeadlessDappError;
use ergo_headless_dapp_framework::{NanoErg, P2PKAddressString, P2SAddressString};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(
        "There are an insufficient number of nanoErgs in total. Required number of nanoErgs: {0}"
    )]
    InsufficientNanoErgs(NanoErg),
    #[error("There are an insufficient number of StableCoins in total. Required number of StableCoins: {0}")]
    InsufficientStableCoins(u64),
    #[error("There are an insufficient number of ReserveCoins in total. Required number of ReserveCoins: {0}")]
    InsufficientReserveCoins(u64),
    #[error(
        "There is an insufficient amount of Base Reserves(Ergs). Required number of Ergs: {0}"
    )]
    InsufficientBaseReserves(NanoErg),
    #[error("The Box value {0} is invalid.")]
    InvalidBoxValue(NanoErg),
    #[error("Invalid P2S Address: {0}")]
    InvalidP2SAddress(P2SAddressString),
    #[error("Invalid P2PK Address: {0}")]
    InvalidP2PKAddress(P2PKAddressString),
    #[error("Invalid Input Value: {0}")]
    InvalidInputValue(String),
    #[error("An `Action` created the following output `Stage` box which failed to pass a predicate: {0}")]
    InvalidOutputStageBox(String),
    #[error("Insufficient Number Of Boxes Provided.")]
    InsufficientNumberOfBoxes(),
    #[error("The Oracle box supplied has an invalid NFT id.")]
    InvalidOracleBoxNFT(),
    #[error("The values attempted to be encoded within registers failed.")]
    InvalidRegisterValues(),
    #[error("The current Reserve Ratio is insufficient to perform the given Action.")]
    InvalidReserveRatio(),
    #[error("Invalid Tokens: {0}")]
    InvalidTokens(String),
    #[error("Invalid Registers: {0}")]
    InvalidRegisters(String),
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    ErgoProtocolFramework(#[from] HeadlessDappError),
}
