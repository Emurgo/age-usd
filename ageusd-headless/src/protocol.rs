use crate::bank::BankBox;
use crate::equations::reserve_ratio;
use crate::error::ProtocolError;
use crate::input_boxes::{ReserveCoinBox, StableCoinBox};
use crate::parameters::{
    BANK_NFT_ID, COOLING_OFF_HEIGHT, IMPLEMENTOR_FEE_PERCENT, MAX_RESERVE_RATIO, MIN_BOX_VALUE,
    MIN_RESERVE_RATIO, RESERVECOIN_DEFAULT_PRICE, RESERVECOIN_TOKEN_ID, STABLECOIN_TOKEN_ID,
};
use crate::receipt::ReceiptBox;
use ergo_headless_dapp_framework::{
    create_candidate, ErgUsdOraclePoolBox, ErgsBox, TokensChangeBox, TxFeeBox, WrappedBox,
};
use ergo_headless_dapp_framework::{BlockHeight, NanoErg};
use ergo_headless_dapp_framework::{ErgoAddressString, P2PKAddressString};
use ergo_lib::chain::transaction::unsigned::UnsignedTransaction;
use ergo_lib::chain::transaction::UnsignedInput;
use ergo_lib_wasm::box_coll::ErgoBoxes;
use ergo_lib_wasm::transaction::UnsignedTransaction as WUnsignedTransaction;
use std::result::Result;
use wasm_bindgen::prelude::*;

/// The struct which represents our multi-stage smart contract protocol
#[wasm_bindgen]
pub struct StableCoinProtocol {}

/// WASM-supported methods related to `StableCoinProtocol`
#[wasm_bindgen]
impl StableCoinProtocol {
    /// Create a new StableCoinProtocol
    #[wasm_bindgen(constructor)]
    pub fn new() -> StableCoinProtocol {
        StableCoinProtocol {}
    }

    #[wasm_bindgen(getter)]
    pub fn min_box_value(&self) -> u64 {
        MIN_BOX_VALUE
    }

    #[wasm_bindgen(getter)]
    pub fn reservecoin_default_price(&self) -> u64 {
        RESERVECOIN_DEFAULT_PRICE
    }

    #[wasm_bindgen(getter)]
    pub fn min_reserve_ratio(&self) -> u64 {
        MIN_RESERVE_RATIO
    }

    #[wasm_bindgen(getter)]
    pub fn max_reserve_ratio(&self) -> u64 {
        MAX_RESERVE_RATIO
    }

    #[wasm_bindgen(getter)]
    pub fn stablecoin_token_id(&self) -> String {
        STABLECOIN_TOKEN_ID.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn reservecoin_token_id(&self) -> String {
        RESERVECOIN_TOKEN_ID.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn bank_nft_id(&self) -> String {
        BANK_NFT_ID.to_string()
    }

    #[wasm_bindgen]
    /// Action: Mint StableCoins by providing Ergs.
    /// This is the WASM wrapper function for said Action.
    pub fn w_action_mint_stablecoin(
        &self,
        amount_to_mint: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        ergo_boxes: &ErgoBoxes,
        implementor_address: ErgoAddressString,
    ) -> Result<WUnsignedTransaction, JsValue> {
        let ergs_boxes: Vec<ErgsBox> = ErgsBox::convert_from_ergo_boxes(ergo_boxes)
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))?;

        let unsigned_tx = self
            .action_mint_stablecoin(
                amount_to_mint.clone(),
                user_address,
                transaction_fee,
                current_height,
                &oracle_box,
                &bank_box,
                &ergs_boxes,
                implementor_address,
            )
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))?;

        Ok(unsigned_tx.into())
    }

    #[wasm_bindgen]
    /// Action: Mint ReserveCoin by providing Ergs.
    /// This is the WASM wrapper function for said Action.
    pub fn w_action_mint_reservecoin(
        &self,
        amount_to_mint: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        ergo_boxes: &ErgoBoxes,
        implementor_address: ErgoAddressString,
    ) -> Result<WUnsignedTransaction, JsValue> {
        let ergs_boxes: Vec<ErgsBox> = ErgsBox::convert_from_ergo_boxes(ergo_boxes)
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))?;

        let unsigned_tx = self
            .action_mint_reservecoin(
                amount_to_mint.clone(),
                user_address,
                transaction_fee,
                current_height,
                &oracle_box,
                &bank_box,
                &ergs_boxes,
                implementor_address,
            )
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))?;

        Ok(unsigned_tx.into())
    }
}

/// Rust methods related to `StableCoinProtocol`
impl StableCoinProtocol {
    /// Action: Mint ReserveCoin by providing Ergs
    pub fn action_mint_reservecoin(
        &self,
        amount_to_mint: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        ergs_boxes: &Vec<ErgsBox>,
        implementor_address: ErgoAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        //
        // Defining useful values
        //
        // Total ergs inside of `ergs_boxes`
        let input_ergs_total = ErgsBox::sum_ergs_boxes_value(&ergs_boxes);
        // Oracle datapoint
        let oracle_rate = oracle_box.datapoint_in_cents();
        // Erg Reserves in Bank Box
        let base_reserves_in = bank_box.base_reserves();
        // Number of ReserveCoins in circulation currently/in inputs
        let circulating_reservecoins_in = bank_box.num_circulating_reservecoins();
        // Amount of Ergs needed to cover amount_to_mint
        let reservecoin_value_in_base =
            bank_box.base_cost_to_mint_reservecoin(amount_to_mint, oracle_box);
        // Amount to pay out implementor.
        let implementor_fee = (reservecoin_value_in_base as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;
        let base_reserves_out = base_reserves_in + reservecoin_value_in_base;
        // New ReserveCoins in circulation after minting
        let circulating_reservecoins_out = circulating_reservecoins_in + amount_to_mint;
        // The new reserve ratio that will be in the output Bank box
        let reserve_ratio_out = reserve_ratio(
            base_reserves_out,
            bank_box.num_circulating_stablecoins(),
            oracle_rate,
        );

        //
        // Performing Checks
        //
        // Ensure Reserve Ratio is below maximum
        if reserve_ratio_out >= self.max_reserve_ratio() && current_height > COOLING_OFF_HEIGHT {
            return Err(ProtocolError::InvalidReserveRatio());
        }
        // Ensure more than 0 ReserveCoins are attempted to be minted
        if amount_to_mint == 0 {
            return Err(ProtocolError::InvalidInputValue(
                "The user must mint at least 1 ReserveCoin.".to_string(),
            ));
        }
        // Verify that at least 1 ErgsBox was provided
        if ergs_boxes.len() == 0 {
            return Err(ProtocolError::InsufficientNumberOfBoxes());
        }
        // Verify that the provided ergs_boxes hold sufficient nanoErgs to
        // cover the minting, the tx fee, and to have MIN_BOX_VALUE in the
        // Receipt box.
        if input_ergs_total
            < (reservecoin_value_in_base + transaction_fee + self.min_box_value() + implementor_fee)
        {
            return Err(ProtocolError::InsufficientNanoErgs(
                reservecoin_value_in_base,
            ));
        }

        //
        // Setting Up The Tx Inputs
        //
        // Define the tx input boxes
        let mut tx_input_boxes = vec![bank_box.get_box()];
        tx_input_boxes.append(&mut ergs_boxes.into_iter().map(|b| b.get_box()).collect());
        // Convert them into `UnsignedInput`s
        let tx_inputs: Vec<UnsignedInput> = tx_input_boxes
            .clone()
            .into_iter()
            .map(|eb| eb.into())
            .collect();
        // Seting up Data-inputs
        let data_inputs = vec![oracle_box.as_data_input()];

        //
        // Setting Up The Output Boxes
        //
        // Create the output bank box candidate
        let output_bank_candidate = bank_box.create_mint_reservecoin_candidate(
            amount_to_mint,
            current_height,
            circulating_reservecoins_out,
            reservecoin_value_in_base,
            &bank_box,
        )?;

        // Create the Receipt box candidate
        let receipt_box_candidate = ReceiptBox::create_mint_reservecoin_candidate(
            amount_to_mint,
            &user_address,
            current_height,
            transaction_fee,
            implementor_fee,
            reservecoin_value_in_base,
            bank_box,
            input_ergs_total,
        )?;

        // Create an output for all of the token change from inputs
        let no_bank_inputs = tx_input_boxes[1..].to_vec();
        let token_change_candidate = TokensChangeBox::output_candidate(
            &no_bank_inputs,
            self.min_box_value(),
            &user_address,
            current_height,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Create the Implementor Fee box candidate
        let implementor_fee_box_candidate = create_candidate(
            implementor_fee,
            &implementor_address,
            &vec![],
            &vec![],
            current_height,
        )?;

        //
        // Creating the UnsignedTransaction
        //
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            data_inputs,
            vec![
                output_bank_candidate,
                receipt_box_candidate,
                token_change_candidate,
                transaction_fee_box_candidate,
                implementor_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }

    /// Action: Mint StableCoin by providing Ergs
    pub fn action_mint_stablecoin(
        &self,
        amount_to_mint: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        ergs_boxes: &Vec<ErgsBox>,
        implementor_address: ErgoAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        //
        // Defining useful values
        //
        // Total ergs inside of `ergs_boxes`
        let input_ergs_total = ErgsBox::sum_ergs_boxes_value(&ergs_boxes);
        // Oracle datapoint
        let oracle_rate = oracle_box.datapoint_in_cents();
        // Erg Reserves in Bank Box
        let base_reserves_in = bank_box.base_reserves();
        // Number of StableCoins in circulation currently/in inputs
        let circulating_stablecoins_in = bank_box.num_circulating_stablecoins();
        // Amount of Ergs needed to cover amount_to_mint
        let stablecoin_value_in_base =
            bank_box.base_cost_to_mint_stablecoin(amount_to_mint, oracle_box);
        // Amount to pay out implementor.
        let implementor_fee = (stablecoin_value_in_base as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;
        // New Base Reserves Total After Minting
        let base_reserves_out = base_reserves_in + stablecoin_value_in_base;
        // New stablecoin in circulation after minting
        let circulating_stablecoins_out = circulating_stablecoins_in + amount_to_mint;
        // The new reserve ratio that will be in the output Bank box
        let reserve_ratio_out =
            reserve_ratio(base_reserves_out, circulating_stablecoins_out, oracle_rate);

        //
        // Performing Checks
        //
        // Ensure Reserve Ratio is above minimum
        if reserve_ratio_out < self.min_reserve_ratio() {
            return Err(ProtocolError::InvalidReserveRatio());
        }
        // Ensure more than 0 StableCoins are attempted to be minted
        if amount_to_mint == 0 {
            return Err(ProtocolError::InvalidInputValue(
                "The user must mint at least 1 StableCoin.".to_string(),
            ));
        }
        // Verify that at least 1 ErgsBox was provided
        if ergs_boxes.len() == 0 {
            return Err(ProtocolError::InsufficientNumberOfBoxes());
        }
        // Verify that the provided ergs_boxes hold sufficient nanoErgs to
        // cover the minting, the tx fee, and to have MIN_BOX_VALUE in the
        // Receipt box.
        if input_ergs_total
            < (stablecoin_value_in_base
                + transaction_fee
                + (self.min_box_value() * 2)
                + implementor_fee)
        {
            return Err(ProtocolError::InsufficientNanoErgs(
                stablecoin_value_in_base,
            ));
        }

        //
        // Setting Up The Tx Inputs
        //
        // Define the tx input boxes
        let mut tx_input_boxes = vec![bank_box.get_box()];
        tx_input_boxes.append(&mut ergs_boxes.into_iter().map(|b| b.get_box()).collect());
        // Convert them into `UnsignedInput`s
        let tx_inputs: Vec<UnsignedInput> = tx_input_boxes
            .clone()
            .into_iter()
            .map(|eb| eb.into())
            .collect();

        // Seting up Data-inputs
        let data_inputs = vec![oracle_box.as_data_input()];

        //
        // Setting Up The Output Boxes
        //
        let output_bank_candidate = bank_box.create_mint_stablecoin_candidate(
            amount_to_mint,
            current_height,
            circulating_stablecoins_out,
            stablecoin_value_in_base,
            &bank_box,
        )?;

        // Create the Receipt box candidate
        let receipt_box_candidate = ReceiptBox::create_mint_stablecoin_candidate(
            amount_to_mint,
            &user_address,
            current_height,
            transaction_fee,
            implementor_fee,
            stablecoin_value_in_base,
            bank_box,
            input_ergs_total,
        )?;

        // Create an output for all of the token change from inputs
        let no_bank_inputs = tx_input_boxes[1..].to_vec();
        let token_change_candidate = TokensChangeBox::output_candidate(
            &no_bank_inputs,
            self.min_box_value(),
            &user_address,
            current_height,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Create the Implementor Fee box candidate
        let implementor_fee_box_candidate = create_candidate(
            implementor_fee,
            &implementor_address,
            &vec![],
            &vec![],
            current_height,
        )?;

        //
        // Creating the UnsignedTransaction
        //
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            data_inputs,
            vec![
                output_bank_candidate,
                receipt_box_candidate,
                token_change_candidate,
                transaction_fee_box_candidate,
                implementor_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }

    /// Action: Redeem ReserveCoin for Ergs
    pub fn action_redeem_reservecoin(
        &self,
        amount_to_redeem: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        rc_boxes: &Vec<ReserveCoinBox>,
        implementor_address: ErgoAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        //
        // Defining useful values
        //
        // The total number of ReserveCoins provided as inputs in rc_boxes
        let input_reservecoins_total = ReserveCoinBox::sum_token_amount(&rc_boxes);
        // Oracle datapoint
        let oracle_rate = oracle_box.datapoint_in_cents();
        // Erg Reserves in Bank Box
        let base_reserves_in = bank_box.base_reserves();
        // Number of ReserveCoins in circulation currently/in inputs
        let circulating_reservecoins_in = bank_box.num_circulating_reservecoins();
        // Amount of Ergs the user will receive based on amount_to_redeem
        let reservecoin_value_in_base =
            bank_box.base_amount_from_redeeming_reservecoin(amount_to_redeem, oracle_box);
        // Amount to pay out implementor.
        let implementor_fee = (reservecoin_value_in_base as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;
        // Check that sufficient number of circulating ReserveCoins

        if circulating_reservecoins_in < amount_to_redeem {
            return Err(ProtocolError::InsufficientReserveCoins(amount_to_redeem));
        }
        // Check that sufficient number of ReserveCoins are circulating
        if reservecoin_value_in_base > base_reserves_in {
            return Err(ProtocolError::InsufficientBaseReserves(base_reserves_in));
        }
        // New Base Reserves Total After Redeeming
        let base_reserves_out = base_reserves_in - reservecoin_value_in_base;
        // New ReserveCoins in circulation after redeeming
        let circulating_reservecoins_out = circulating_reservecoins_in - amount_to_redeem;
        // The new reserve ratio that will be in the output Bank box
        let mut reserve_ratio_out = reserve_ratio(
            base_reserves_out,
            bank_box.num_circulating_stablecoins(),
            oracle_rate,
        );
        // If number of circulating stablecoins == 0 then set reserve ratio
        // to max in order to allow redeeming.
        if bank_box.num_circulating_stablecoins() == 0 {
            reserve_ratio_out = self.max_reserve_ratio();
        }

        //
        // Performing Checks
        //
        // // Ensure output Reserve Ratio is above minimum
        if reserve_ratio_out <= self.min_reserve_ratio() {
            return Err(ProtocolError::InvalidReserveRatio());
        }
        // Ensure more than 0 ReserveCoins are attempted to be redeemed
        if amount_to_redeem == 0 {
            return Err(ProtocolError::InvalidInputValue(
                "The user must redeem at least 1 ReserveCoin.".to_string(),
            ));
        }
        // Verify that at least 1 ErgsBox was provided
        if rc_boxes.len() == 0 {
            return Err(ProtocolError::InsufficientNumberOfBoxes());
        }
        // Verify that the provided rc_boxes hold sufficient ReserveCoins to
        // cover the redeeming.
        if input_reservecoins_total < amount_to_redeem {
            return Err(ProtocolError::InsufficientReserveCoins(amount_to_redeem));
        }

        //
        // Setting Up The Tx Inputs
        //

        // Define the tx input boxes
        let mut tx_input_boxes = vec![bank_box.get_box()];
        tx_input_boxes.append(&mut rc_boxes.into_iter().map(|b| b.get_box()).collect());
        // Convert them into `UnsignedInput`s
        let tx_inputs: Vec<UnsignedInput> = tx_input_boxes
            .clone()
            .into_iter()
            .map(|eb| eb.into())
            .collect();

        // Seting up Data-inputs
        let data_inputs = vec![oracle_box.as_data_input()];

        //
        // Setting Up The Output Boxes
        //
        let output_bank_candidate = bank_box.create_redeem_reservecoin_candidate(
            amount_to_redeem,
            current_height,
            circulating_reservecoins_out,
            reservecoin_value_in_base,
            &bank_box,
        )?;

        // The Receipt box
        let no_bank_inputs = tx_input_boxes[1..].to_vec();
        let receipt_box_candidate = ReceiptBox::create_redeem_reservecoin_candidate(
            amount_to_redeem,
            &user_address,
            current_height,
            transaction_fee,
            reservecoin_value_in_base,
            bank_box,
            rc_boxes,
            &no_bank_inputs,
            implementor_fee,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Create the Implementor Fee box candidate
        let implementor_fee_box_candidate = create_candidate(
            implementor_fee,
            &implementor_address,
            &vec![],
            &vec![],
            current_height,
        )?;

        //
        // Creating the UnsignedTransaction
        //
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            data_inputs,
            vec![
                output_bank_candidate,
                receipt_box_candidate,
                transaction_fee_box_candidate,
                implementor_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }

    /// Action: Redeem StableCoin for Ergs
    pub fn action_redeem_stablecoin(
        &self,
        amount_to_redeem: u64,
        user_address: P2PKAddressString,
        transaction_fee: NanoErg,
        current_height: BlockHeight,
        oracle_box: &ErgUsdOraclePoolBox,
        bank_box: &BankBox,
        sc_boxes: &Vec<StableCoinBox>,
        implementor_address: ErgoAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        //
        // Defining useful values
        //
        // The total number of StableCoins provided as inputs in rc_boxes
        let input_stablecoins_total = StableCoinBox::sum_token_amount(&sc_boxes);
        // Oracle datapoint
        let oracle_rate = oracle_box.datapoint_in_cents();
        // Base Reserves of Bank
        let base_reserves_in = bank_box.base_reserves();
        // Number of StableCoins in circulation currently/in inputs
        let circulating_stablecoins_in = bank_box.num_circulating_stablecoins();
        // Amount of Ergs the user will receive based on amount_to_redeem
        let stablecoin_value_in_base =
            bank_box.base_amount_from_redeeming_stablecoin(amount_to_redeem, oracle_box);
        // Amount to pay out implementor.
        let implementor_fee = (stablecoin_value_in_base as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;

        // Check that sufficient number of StableCoins are circulating
        if circulating_stablecoins_in < amount_to_redeem {
            return Err(ProtocolError::InsufficientStableCoins(amount_to_redeem));
        }
        // Check that sufficient reserves exist
        if stablecoin_value_in_base > base_reserves_in {
            return Err(ProtocolError::InsufficientBaseReserves(base_reserves_in));
        }
        // New StableCoins in circulation after redeeming
        let circulating_stablecoins_out = circulating_stablecoins_in - amount_to_redeem;

        //
        // Performing Further Checks
        //
        // Ensure more than 0 StableCoins are attempted to be redeemed
        if amount_to_redeem == 0 {
            return Err(ProtocolError::InvalidInputValue(
                "The user must redeem at least 1 StableCoin.".to_string(),
            ));
        }
        // Verify that at least 1 ErgsBox was provided
        if sc_boxes.len() == 0 {
            return Err(ProtocolError::InsufficientNumberOfBoxes());
        }
        // Verify that the provided sc_boxes hold sufficient StableCoins to
        // cover the redeeming.
        if input_stablecoins_total < amount_to_redeem {
            return Err(ProtocolError::InsufficientStableCoins(amount_to_redeem));
        }

        //
        // Setting Up The Tx Inputs
        //
        // Define the tx input boxes
        let mut tx_input_boxes = vec![bank_box.get_box()];
        tx_input_boxes.append(&mut sc_boxes.into_iter().map(|scb| scb.get_box()).collect());
        // Convert them into `UnsignedInput`s
        let tx_inputs: Vec<UnsignedInput> = tx_input_boxes
            .clone()
            .into_iter()
            .map(|eb| eb.into())
            .collect();

        // Seting up Data-inputs
        let data_inputs = vec![oracle_box.as_data_input()];

        //
        // Setting Up The Output Boxes
        //
        let output_bank_candidate = bank_box.create_redeem_stablecoin_candidate(
            amount_to_redeem,
            current_height,
            circulating_stablecoins_out,
            stablecoin_value_in_base,
            &bank_box,
        )?;

        // The Receipt box
        let no_bank_inputs = tx_input_boxes[1..].to_vec();
        let receipt_box_candidate = ReceiptBox::create_redeem_stablecoin_candidate(
            amount_to_redeem,
            &user_address,
            current_height,
            transaction_fee,
            stablecoin_value_in_base,
            bank_box,
            sc_boxes,
            &no_bank_inputs,
            implementor_fee,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Create the Implementor Fee box candidate
        let implementor_fee_box_candidate = create_candidate(
            implementor_fee,
            &implementor_address,
            &vec![],
            &vec![],
            current_height,
        )?;

        //
        // Creating the UnsignedTransaction
        //
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            data_inputs,
            vec![
                output_bank_candidate,
                receipt_box_candidate,
                transaction_fee_box_candidate,
                implementor_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }
}
