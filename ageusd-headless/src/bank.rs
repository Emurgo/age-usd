// This file specifies the Bank stage (aka. the BankBox).
// The BankBox implements methods for acquiring it via a `BoxSpec` interface,
// methods for reading current state of the protocol, as well as
// methods for building an output box (creating `ErgoBoxCandidate`s for
// Actions within the protocol).
use crate::equations::reserve_ratio;
use crate::error::ProtocolError;
use crate::parameters::{
    BANK_NFT_ID, FEE_PERCENT, IMPLEMENTOR_FEE_PERCENT, MIN_BOX_VALUE, RESERVECOIN_DEFAULT_PRICE,
    RESERVECOIN_TOKEN_ID, STABLECOIN_TOKEN_ID,
};
use ergo_headless_dapp_framework::{
    create_candidate, BoxSpec, ExplorerFindable, HeadlessDappError, SpecBox, SpecifiedBox,
    TokenSpec, WrapBox, WrappedBox,
};
use ergo_headless_dapp_framework::{encoding::unwrap_long, ErgUsdOraclePoolBox};
use ergo_headless_dapp_framework::{BlockHeight, NanoErg, P2SAddressString};
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::token::{Token, TokenAmount};
use ergo_lib::chain::transaction::TxId;
use ergo_lib_wasm::ergo_box::ErgoBox as WErgoBox;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

/// The struct which represents the `Bank` stage.
#[wasm_bindgen]
#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct BankBox {
    ergo_box: ErgoBox,
}

impl SpecifiedBox for BankBox {
    /// A `BoxSpec` that checks that the box is a valid BankBox via
    /// looking that it holds the correct StableCoin/ReserveCoin tokens
    /// and the Bank NFT.
    fn box_spec() -> BoxSpec {
        let tok_1_spec = Some(TokenSpec::new(1..u64::MAX, STABLECOIN_TOKEN_ID));
        let tok_2_spec = Some(TokenSpec::new(1..u64::MAX, RESERVECOIN_TOKEN_ID));
        let tok_3_spec = Some(TokenSpec::new(1..2, BANK_NFT_ID));
        let tok_specs = vec![tok_1_spec, tok_2_spec, tok_3_spec];
        BoxSpec::new(None, None, vec![], tok_specs)
    }
}

/// WASM-supported methods related to `BankStage`
#[wasm_bindgen]
impl BankBox {
    /// Create a new BankBox via WASM
    #[wasm_bindgen(constructor)]
    pub fn w_new(ergo_box: WErgoBox) -> Result<BankBox, JsValue> {
        let b: ErgoBox = ergo_box.into();
        Self::box_spec()
            .verify_box(&b)
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))?;
        Ok(BankBox {
            ergo_box: b.clone(),
        })
    }

    /// Acquire the current Reserve Ratio in the Bank box
    #[wasm_bindgen]
    pub fn current_reserve_ratio(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        reserve_ratio(
            self.base_reserves(),
            self.num_circulating_stablecoins(),
            oracle_box.datapoint_in_cents(),
        )
    }

    /// Provides the base(Erg) reserves of the Bank. This is the total amount
    /// of nanoErgs held inside, minus the minimum box value required for
    /// posting a box on-chain.
    #[wasm_bindgen]
    pub fn base_reserves(&self) -> NanoErg {
        if self.ergo_box.value.as_u64().clone() < MIN_BOX_VALUE {
            return 0;
        }
        // self.ergo_box.value.as_u64().clone() - MIN_BOX_VALUE
        self.ergo_box.value.as_u64().clone()
    }

    /// Outstanding liabilities in `NanoErg`s to cover the current minted
    /// StableCoins (StableCoins in circulation)
    #[wasm_bindgen]
    pub fn liabilities(&self, oracle_box: &ErgUsdOraclePoolBox) -> NanoErg {
        if self.num_circulating_stablecoins() == 0 {
            return 0;
        } else {
            // The true liabilities for outstanding StableCoins
            let base_reserves_needed =
                self.num_circulating_stablecoins() * oracle_box.datapoint_in_cents();
            // Returns the minimum between the reserves and the true liabilities
            // to cover the scenario where reserves are not sufficient.
            return std::cmp::min(self.base_reserves(), base_reserves_needed);
        }
    }

    /// The equity of the protocol. In other words what base reserves are left
    /// after having covered all liabilities.
    #[wasm_bindgen]
    pub fn equity(&self, oracle_box: &ErgUsdOraclePoolBox) -> NanoErg {
        if self.base_reserves() < self.liabilities(oracle_box) {
            return 0;
        }
        self.base_reserves() - self.liabilities(oracle_box)
    }

    /// The number of StableCoins currently minted. In other words the number
    /// currently in circulation. Held in R4 of Bank box.
    #[wasm_bindgen]
    pub fn num_circulating_stablecoins(&self) -> u64 {
        let registers = self.registers();
        // Using unwrap because the `StageBox<Bank>` check should guarantee
        // we have a valid box at the `Bank` stage.
        unwrap_long(&registers[0]).unwrap() as u64
    }

    /// The number of ReserveCoins currently minted. In other words the number
    /// currently in circulation. Held in R5 of Bank box.
    #[wasm_bindgen]
    pub fn num_circulating_reservecoins(&self) -> u64 {
        let registers = self.registers();
        // Using unwrap because the `StageBox<Bank>` check should guarantee
        // we have a valid box at the `Bank` stage.
        unwrap_long(&registers[1]).unwrap() as u64
    }

    /// Current StableCoin nominal price
    #[wasm_bindgen]
    pub fn stablecoin_nominal_price(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        let oracle_rate = oracle_box.datapoint_in_cents();
        if self.num_circulating_stablecoins() == 0
            || oracle_rate < self.liabilities(oracle_box) / self.num_circulating_stablecoins()
        {
            return oracle_rate;
        } else {
            return self.liabilities(oracle_box) / self.num_circulating_stablecoins();
        }
    }

    /// Current ReserveCoin nominal price
    #[wasm_bindgen]
    pub fn reservecoin_nominal_price(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        if self.num_circulating_reservecoins() == 0 || self.equity(oracle_box) == 0 {
            return RESERVECOIN_DEFAULT_PRICE;
        }
        self.equity(oracle_box) / self.num_circulating_reservecoins()
    }

    /// The total amount of nanoErgs which is needed to cover minting
    /// the provided number of ReserveCoins, cover tx fees, implementor
    /// fee, etc.
    pub fn total_cost_to_mint_stablecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let base_cost = self.base_cost_to_mint_stablecoin(amount_to_mint, &oracle_box);
        base_cost
            + transaction_fee
            + (MIN_BOX_VALUE * 2)
            + (base_cost as f64 * IMPLEMENTOR_FEE_PERCENT) as u64
    }

    /// The amount of base currency (Ergs) which is needed to cover minting
    /// the provided number of StableCoins.
    #[wasm_bindgen]
    pub fn base_cost_to_mint_stablecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.stablecoin_nominal_price(oracle_box) * amount_to_mint;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        feeless_cost + protocol_fee
    }

    /// The total amount of nanoErgs which is needed to cover minting
    /// the provided number of ReserveCoins, cover tx fees, implementor
    /// fee, etc.
    pub fn total_cost_to_mint_reservecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let base_cost = self.base_cost_to_mint_reservecoin(amount_to_mint, &oracle_box);
        base_cost
            + transaction_fee
            + (MIN_BOX_VALUE * 2)
            + (base_cost as f64 * IMPLEMENTOR_FEE_PERCENT) as u64
    }

    /// The amount of base currency (Ergs) which is needed to cover minting
    /// the provided number of ReserveCoins.
    #[wasm_bindgen]
    pub fn base_cost_to_mint_reservecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.reservecoin_nominal_price(oracle_box) * amount_to_mint;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        feeless_cost + protocol_fee
    }

    /// The amount of nanoErgs which will be redeemed
    /// from the protocol based on current reserves + the number of
    /// ReserveCoins being redeemed by the user after paying for
    // tx fees, implementor fees, etc.
    #[wasm_bindgen]
    pub fn total_amount_from_redeeming_reservecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.reservecoin_nominal_price(oracle_box) * amount_to_redeem;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        let base_cost = feeless_cost + protocol_fee;

        base_cost - transaction_fee - (base_cost as f64 * IMPLEMENTOR_FEE_PERCENT) as u64
    }

    /// The amount of base currency (Ergs) which will be redeemed
    /// from the protocol based on current reserves + the number of
    /// ReserveCoins being redeemed by the user.
    #[wasm_bindgen]
    pub fn base_amount_from_redeeming_reservecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.reservecoin_nominal_price(oracle_box) * amount_to_redeem;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        feeless_cost - protocol_fee
    }

    /// The amount of nanoErgs which will be redeemed
    /// from the protocol based on current reserves + the number of
    /// StableCoins being redeemed by the user after paying for
    // tx fees, implementor fees, etc.
    #[wasm_bindgen]
    pub fn total_amount_from_redeeming_stablecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.stablecoin_nominal_price(oracle_box) * amount_to_redeem;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        let base_cost = feeless_cost + protocol_fee;

        base_cost - transaction_fee - (base_cost as f64 * IMPLEMENTOR_FEE_PERCENT) as u64
    }

    /// The amount of base currency (Ergs) which will be redeemed
    /// from the protocol based on current reserves + the number of
    /// StableCoins being redeemed by the user.
    #[wasm_bindgen]
    pub fn base_amount_from_redeeming_stablecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_cost = self.stablecoin_nominal_price(oracle_box) * amount_to_redeem;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_cost * FEE_PERCENT / 100;

        feeless_cost - protocol_fee
    }
}

/// Rust methods related to `BankStage`
impl BankBox {
    /// Create an `ErgoBoxCandidate` for the output Bank box for the
    /// `Mint ReserveCoin` action
    pub fn create_mint_reservecoin_candidate(
        &self,
        amount_to_mint: u64,
        current_height: BlockHeight,
        circulating_reservecoins_out: u64,
        reservecoin_value_in_base: NanoErg,
        input_bank_box: &BankBox,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Specify the tokens in the Bank Box
        let stablecoin_tokens = self.tokens()[0].clone();
        // Specify ReserveCoins
        let bank_reservecoin_token_in = self.tokens()[1].clone();
        let token_amount =
            TokenAmount::try_from(u64::from(bank_reservecoin_token_in.amount) - amount_to_mint)
                .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
        let reservecoin_tokens = Token {
            token_id: bank_reservecoin_token_in.token_id.clone(),
            amount: token_amount,
        };
        let nft_token = self.tokens()[2].clone();
        let obb_tokens = vec![stablecoin_tokens, reservecoin_tokens, nft_token];
        // Specify the registers in the Bank Box
        let registers_vec = vec![
            (self.num_circulating_stablecoins() as i64).into(),
            (circulating_reservecoins_out as i64).into(),
        ];
        // Creating the output Bank box candidate
        let output_bank_candidate = create_candidate(
            self.nano_ergs() + reservecoin_value_in_base,
            &input_bank_box.p2s_address(),
            &obb_tokens,
            &registers_vec,
            current_height,
        )?;

        // Verify the output Bank box is a valid Bank box
        let processed_box = &ErgoBox::from_box_candidate(&output_bank_candidate, TxId::zero(), 0);
        BankBox::box_spec().verify_box(processed_box)?;

        Ok(output_bank_candidate)
    }

    /// Create an `ErgoBoxCandidate` for the output Bank box for the
    /// `Mint StableCoin` action
    pub fn create_mint_stablecoin_candidate(
        &self,
        amount_to_mint: u64,
        current_height: BlockHeight,
        circulating_stablecoins_out: u64,
        stablecoin_value_in_base: u64,
        input_bank_box: &BankBox,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let bank_stablecoin_token_in = self.tokens()[0].clone();
        let token_amount =
            TokenAmount::try_from(u64::from(bank_stablecoin_token_in.amount) - amount_to_mint)
                .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
        let stablecoin_tokens = Token {
            token_id: bank_stablecoin_token_in.token_id.clone(),
            amount: token_amount,
        };
        let reservecoin_tokens = self.tokens()[1].clone();
        let nft_token = self.tokens()[2].clone();
        let obb_tokens = vec![stablecoin_tokens, reservecoin_tokens, nft_token];
        // Specify the registers in the Bank Box
        let registers_vec = vec![
            (circulating_stablecoins_out as i64).into(),
            (self.num_circulating_reservecoins() as i64).into(),
        ];
        // Creating the output Bank box candidate
        let output_bank_candidate = create_candidate(
            self.nano_ergs() + stablecoin_value_in_base,
            &input_bank_box.p2s_address(),
            &obb_tokens,
            &registers_vec,
            current_height,
        )?;

        // Verify the output Bank box is a valid Bank box
        let processed_box = &ErgoBox::from_box_candidate(&output_bank_candidate, TxId::zero(), 0);
        BankBox::box_spec().verify_box(processed_box)?;

        Ok(output_bank_candidate)
    }

    /// Create an `ErgoBoxCandidate` for the output Bank box for the
    /// `Redeem ReserveCoin` action
    pub fn create_redeem_reservecoin_candidate(
        &self,
        amount_to_redeem: u64,
        current_height: BlockHeight,
        circulating_reservecoins_out: u64,
        reservecoin_value_in_base: u64,
        input_bank_box: &BankBox,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Specify the tokens in the Bank Box
        let stablecoin_tokens = self.tokens()[0].clone();
        // Specifying ReserveCoins
        let bank_reservecoin_token_in = self.tokens()[1].clone();
        let token_amount =
            TokenAmount::try_from(u64::from(bank_reservecoin_token_in.amount) + amount_to_redeem)
                .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
        let reservecoin_tokens = Token {
            token_id: bank_reservecoin_token_in.token_id.clone(),
            amount: token_amount,
        };
        let nft_token = self.tokens()[2].clone();
        let obb_tokens = vec![stablecoin_tokens, reservecoin_tokens, nft_token];
        // Specify the registers in the Bank Box
        let registers_vec = vec![
            (self.num_circulating_stablecoins() as i64).into(),
            (circulating_reservecoins_out as i64).into(),
        ];
        // Creating the output Bank box candidate
        let output_bank_candidate = create_candidate(
            self.nano_ergs() - reservecoin_value_in_base,
            &input_bank_box.p2s_address(),
            &obb_tokens,
            &registers_vec,
            current_height,
        )?;

        // Verify the output Bank box is a valid Bank box
        let processed_box = &ErgoBox::from_box_candidate(&output_bank_candidate, TxId::zero(), 0);
        BankBox::box_spec().verify_box(processed_box)?;

        Ok(output_bank_candidate)
    }

    /// Create an `ErgoBoxCandidate` for the output Bank box for the
    /// `Redeem StableCoin` action
    pub fn create_redeem_stablecoin_candidate(
        &self,
        amount_to_redeem: u64,
        current_height: BlockHeight,
        circulating_stablecoins_out: u64,
        stablecoin_value_in_base: u64,
        input_bank_box: &BankBox,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let bank_stablecoin_token_in = self.tokens()[0].clone();
        let token_amount =
            TokenAmount::try_from(u64::from(bank_stablecoin_token_in.amount) + amount_to_redeem)
                .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
        let stablecoin_tokens = Token {
            token_id: bank_stablecoin_token_in.token_id.clone(),
            amount: token_amount,
        };
        let reservecoin_tokens = self.tokens()[1].clone();
        let nft_token = self.tokens()[2].clone();
        let obb_tokens = vec![stablecoin_tokens, reservecoin_tokens, nft_token];
        // Specify the registers in the Bank Box
        let registers_vec = vec![
            (circulating_stablecoins_out as i64).into(),
            (self.num_circulating_reservecoins() as i64).into(),
        ];
        // Creating the output Bank box candidate
        let output_bank_candidate = create_candidate(
            self.nano_ergs() - stablecoin_value_in_base,
            &input_bank_box.p2s_address(),
            &obb_tokens,
            &registers_vec,
            current_height,
        )?;

        // Verify the output Bank box is a valid Bank box
        let processed_box = &ErgoBox::from_box_candidate(&output_bank_candidate, TxId::zero(), 0);
        BankBox::box_spec().verify_box(processed_box)?;

        Ok(output_bank_candidate)
    }

    /// Create an `ErgoBoxCandidate` for the output Bank Box for the
    /// `Update Protocol` Action
    pub fn create_update_protocol_candidate(
        &self,
        update_address: &P2SAddressString,
        input_bank_box: &BankBox,
        current_height: BlockHeight,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let output_bank_candidate = create_candidate(
            input_bank_box.nano_ergs(),
            update_address,
            &input_bank_box.tokens(),
            &input_bank_box.registers(),
            current_height,
        )?;
        Ok(output_bank_candidate)
    }
}
