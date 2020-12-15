/// This file holds structs that implement `SpecifiedBox` which are to be
/// used as inputs to Actions.
use crate::error::{ProtocolError, Result};
use crate::parameters::{RESERVECOIN_TOKEN_ID, STABLECOIN_TOKEN_ID};
pub use ergo_headless_dapp_framework::box_traits::{ExplorerFindable, SpecifiedBox, WrappedBox};
pub use ergo_headless_dapp_framework::specified_boxes::{ErgUsdOraclePoolBox, ErgsBox};
use ergo_headless_dapp_framework::{BoxSpec, HeadlessDappError, SpecBox, WrapBox};
use ergo_lib::chain::ergo_box::ErgoBox;
use ergo_lib_wasm::ergo_box::ErgoBox as WErgoBox;
use wasm_bindgen::prelude::*;

/// A predicated box which holds ReserveCoins
#[wasm_bindgen]
#[derive(Clone, Debug, WrapBox, SpecBox)]
pub struct ReserveCoinBox {
    ergo_box: ErgoBox,
}

/// WASM ReserveCoinBox Methods
#[wasm_bindgen]
impl ReserveCoinBox {
    /// Create a new `ReserveCoin`
    #[wasm_bindgen(constructor)]
    pub fn w_new(wb: WErgoBox) -> std::result::Result<ReserveCoinBox, JsValue> {
        let b: ErgoBox = wb.into();
        ReserveCoinBox::new(&b).map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
    }

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
}

/// A predicated box which holds StableCoins
#[wasm_bindgen]
#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct StableCoinBox {
    ergo_box: ErgoBox,
}

/// WASM StableCoinBox Methods
#[wasm_bindgen]
impl StableCoinBox {
    /// Create a new `StableCoinBox`
    #[wasm_bindgen(constructor)]
    pub fn w_new(wb: WErgoBox) -> std::result::Result<StableCoinBox, JsValue> {
        let b: ErgoBox = wb.into();
        StableCoinBox::new(&b).map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
    }

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
}
