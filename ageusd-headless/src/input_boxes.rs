/// This file holds structs that implement `SpecifiedBox` which are to be
/// used as inputs to Actions.
use crate::error::{ProtocolError, Result};
use crate::parameters::{RESERVECOIN_TOKEN_ID, STABLECOIN_TOKEN_ID};
pub use ergo_headless_dapp_framework::box_traits::{ExplorerFindable, SpecifiedBox, WrappedBox};
pub use ergo_headless_dapp_framework::specified_boxes::{ErgUsdOraclePoolBox, ErgsBox};
use ergo_headless_dapp_framework::{BoxSpec, HeadlessDappError, SpecBox, WASMBox, WrapBox};
use ergo_lib::chain::ergo_box::ErgoBox;
use ergo_lib_wasm::ergo_box::ErgoBox as WErgoBox;
use ergo_lib_wasm::box_coll::ErgoBoxes;
use wasm_bindgen::prelude::*;

/// A predicated box which holds ReserveCoins
#[wasm_bindgen]
#[derive(Debug, Clone, WrapBox, SpecBox, WASMBox)]
pub struct ReserveCoinBox {
    ergo_box: ErgoBox,
}

/// WASM ReserveCoinBox Methods
#[wasm_bindgen]
impl ReserveCoinBox {
    /// Get the amount of tokens within the box
    #[wasm_bindgen(getter)]
    pub fn token_amount(&self) -> u64 {
        Self::extract_token_amount(&self.get_box()).unwrap()
    }
}

/// SpecifiedBox impl
impl SpecifiedBox for ReserveCoinBox {
    /// A `BoxSpec` that checks that ReserveCoins are in the box via a
    /// predicate.
    fn box_spec() -> BoxSpec {
        BoxSpec::new_predicated(None, None, vec![], vec![], Some(Self::predicate))
    }
}

/// Rust ReserveCoinBox Methods
impl ReserveCoinBox {
    /// Predicate to check that the box has ReserveCoins in it
    fn predicate(b: &ErgoBox) -> bool {
        Self::extract_token_amount(b).is_ok()
    }

    /// Acquires the token amount
    fn extract_token_amount(b: &ErgoBox) -> Result<u64> {
        for t in &b.tokens {
            let token_id_string: String = t.token_id.0.clone().into();
            if token_id_string == RESERVECOIN_TOKEN_ID {
                return Ok(u64::from(t.amount));
            }
        }
        Err(ProtocolError::InvalidTokens(
            "No ReserveCoins found in box.".to_string(),
        ))
    }

    /// Sums the nanoErg value of a list of `ReserveCoinBox`es
    pub fn sum_nano_ergs_value(boxes: &Vec<ReserveCoinBox>) -> u64 {
        boxes.into_iter().fold(0, |acc, pb| pb.nano_ergs() + acc)
    }

    /// Sums the token amount of a list of `ReserveCoinBox`es
    pub fn sum_token_amount(boxes: &Vec<ReserveCoinBox>) -> u64 {
        boxes.into_iter().fold(0, |acc, b| b.token_amount() + acc)
    }

    /// Converts from the WASM wrapper `ErgoBoxes`.
    pub fn convert_from_ergo_boxes(ergo_boxes: &ErgoBoxes) -> Result<Vec<ReserveCoinBox>> {
        let mut boxes: Vec<ReserveCoinBox> = vec![];
        let unwrapped_boxes: Vec<ErgoBox> = ergo_boxes.clone().into();
        for b in unwrapped_boxes {
            let ergs_box = ReserveCoinBox::new(&b)?;
            boxes.push(ergs_box);
        }
        Ok(boxes)
    }
}

/// A predicated box which holds StableCoins
#[wasm_bindgen]
#[derive(Debug, Clone, WrapBox, SpecBox, WASMBox)]
pub struct StableCoinBox {
    ergo_box: ErgoBox,
}

/// WASM StableCoinBox Methods
#[wasm_bindgen]
impl StableCoinBox {
    #[wasm_bindgen(getter)]
    /// Get the amount of tokens within the box
    pub fn token_amount(&self) -> u64 {
        Self::extract_token_amount(&self.get_box()).unwrap()
    }
}

/// SpecifiedBox impl
impl SpecifiedBox for StableCoinBox {
    /// A `BoxSpec` that checks that StableCoins are in the box via a
    /// predicate.
    fn box_spec() -> BoxSpec {
        BoxSpec::new_predicated(None, None, vec![], vec![], Some(Self::predicate))
    }
}

/// Rust StableCoinBox Methods
impl StableCoinBox {
    /// Predicate to check that the box has StableCoins in it
    fn predicate(b: &ErgoBox) -> bool {
        Self::extract_token_amount(b).is_ok()
    }

    /// Acquires the token amount
    fn extract_token_amount(b: &ErgoBox) -> Result<u64> {
        for t in &b.tokens {
            let token_id_string: String = t.token_id.0.clone().into();
            if token_id_string == STABLECOIN_TOKEN_ID {
                return Ok(u64::from(t.amount));
            }
        }
        Err(ProtocolError::InvalidTokens(
            "No StableCoins found in box.".to_string(),
        ))
    }

    /// Sums the nanoErg value of a list of `StableCoinBox`es
    pub fn sum_nano_ergs_value(boxes: &Vec<StableCoinBox>) -> u64 {
        boxes.into_iter().fold(0, |acc, pb| pb.nano_ergs() + acc)
    }

    /// Sums the token amount of a list of `StableCoinBox`es
    pub fn sum_token_amount(boxes: &Vec<StableCoinBox>) -> u64 {
        boxes.into_iter().fold(0, |acc, b| b.token_amount() + acc)
    }

    /// Converts from the WASM wrapper `ErgoBoxes`.
    pub fn convert_from_ergo_boxes(ergo_boxes: &ErgoBoxes) -> Result<Vec<StableCoinBox>> {
        let mut boxes: Vec<StableCoinBox> = vec![];
        let unwrapped_boxes: Vec<ErgoBox> = ergo_boxes.clone().into();
        for b in unwrapped_boxes {
            let ergs_box = StableCoinBox::new(&b)?;
            boxes.push(ergs_box);
        }
        Ok(boxes)
    }
}
