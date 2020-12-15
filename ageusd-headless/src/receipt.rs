// This file specifies an output builder for the Receipt stage.
use crate::bank::BankBox;
use crate::error::ProtocolError;
use crate::input_boxes::{ReserveCoinBox, StableCoinBox};
use crate::parameters::MIN_BOX_VALUE;
use ergo_headless_dapp_framework::{
    create_candidate, find_and_sum_other_tokens, WrapBox, WrappedBox,
};
use ergo_headless_dapp_framework::{BlockHeight, NanoErg, P2PKAddressString, TokensChangeBox};
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::token::{Token, TokenAmount};
use std::convert::TryFrom;

/// The struct which represents the `Receipt` stage.
#[derive(Debug, Clone, WrapBox)]
pub struct ReceiptBox {
    ergo_box: ErgoBox,
}
impl ReceiptBox {
    /// Create an `ErgoBoxCandidate` for an output Receipt box for the
    /// `Mint ReserveCoin` action
    pub fn create_mint_reservecoin_candidate(
        amount_to_mint: u64,
        user_address: &P2PKAddressString,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        implementor_fee: NanoErg,
        reservecoin_value_in_base: NanoErg,
        bank_box: &BankBox,
        input_ergs_total: NanoErg,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Define the ReserveCoin token
        let rb_reservecoin_token = new_reservecoin_token(amount_to_mint, bank_box)?;
        // Define the Receipt Box tokens
        let rb_tokens = vec![rb_reservecoin_token];

        // Specify the registers in the Receipt box
        let rb_registers_vec = vec![
            (amount_to_mint as i64).into(),
            (reservecoin_value_in_base as i64).into(),
        ];

        // Create the Receipt box candidate
        let candidate = create_candidate(
            input_ergs_total
                - reservecoin_value_in_base
                - transaction_fee
                - implementor_fee
                - MIN_BOX_VALUE,
            &user_address,
            &rb_tokens,
            &rb_registers_vec,
            current_height,
        )?;
        Ok(candidate)
    }

    /// Create an `ErgoBoxCandidate` for an output Receipt box for the
    /// `Mint StableCoin` action
    pub fn create_mint_stablecoin_candidate(
        amount_to_mint: u64,
        user_address: &P2PKAddressString,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        implementor_fee: NanoErg,
        stablecoin_value_in_base: NanoErg,
        bank_box: &BankBox,
        input_ergs_total: NanoErg,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Define the StableCoin token
        let rb_stablecoin_token = new_stablecoin_token(amount_to_mint, bank_box)?;
        // Define the Receipt Box tokens
        let rb_tokens = vec![rb_stablecoin_token];

        // Specify the registers in the Receipt box
        let rb_registers_vec = vec![
            (amount_to_mint as i64).into(),
            (stablecoin_value_in_base as i64).into(),
        ];

        // Create the Receipt box candidate
        let candidate = create_candidate(
            input_ergs_total
                - stablecoin_value_in_base
                - transaction_fee
                - implementor_fee
                - MIN_BOX_VALUE,
            &user_address,
            &rb_tokens,
            &rb_registers_vec,
            current_height,
        )?;
        Ok(candidate)
    }

    /// Create an `ErgoBoxCandidate` for an output Receipt box for the
    /// `Redeem ReserveCoin` action
    pub fn create_redeem_reservecoin_candidate(
        amount_to_redeem: u64,
        user_address: &P2PKAddressString,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        reservecoin_value_in_base: NanoErg,
        bank_box: &BankBox,
        rc_boxes: &Vec<ReserveCoinBox>,
        no_bank_inputs: &Vec<ErgoBox>,
        implementor_fee: NanoErg,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Find how many nanoErgs are inside of the ReserveCoin boxes
        let rc_boxes_value = ReserveCoinBox::sum_nano_ergs_value(&rc_boxes);
        // Find how many StableCoins are inside of the StableCoin boxes
        let rc_boxes_total_rc = ReserveCoinBox::sum_token_amount(rc_boxes);

        // The amount of nanoErgs in the rc_boxes + the value of the
        // ReserveCoins being redeemed - the transaction fee
        let rb_value =
            rc_boxes_value + reservecoin_value_in_base - transaction_fee - implementor_fee;

        // Specify the registers in the Receipt box
        let rb_registers_vec = vec![
            (0 - amount_to_redeem as i64).into(),
            (0 - reservecoin_value_in_base as i64).into(),
        ];

        // Define the tokens
        let mut rb_tokens = vec![];
        // Check if there are any extra tokens that aren't being redeemed
        // and include them in the output
        if rc_boxes_total_rc > amount_to_redeem {
            // Define the StableCoin token
            let amount = rc_boxes_total_rc - amount_to_redeem;
            let new_rc_token = new_reservecoin_token(amount, bank_box)?;
            rb_tokens.push(new_rc_token)
        }
        // Find all other tokens held in user-provided input boxes
        let mut other_tokens =
            find_and_sum_other_tokens(&vec![bank_box.tokens()[1].clone()], &no_bank_inputs);
        rb_tokens.append(&mut other_tokens);

        let candidate = create_candidate(
            rb_value,
            &user_address,
            &rb_tokens,
            &rb_registers_vec,
            current_height,
        )?;
        Ok(candidate)
    }

    /// Create an `ErgoBoxCandidate` for an output Receipt box for the
    /// `Redeem StableCoin` action
    pub fn create_redeem_stablecoin_candidate(
        amount_to_redeem: u64,
        user_address: &P2PKAddressString,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        stablecoin_value_in_base: NanoErg,
        bank_box: &BankBox,
        sc_boxes: &Vec<StableCoinBox>,
        no_bank_inputs: &Vec<ErgoBox>,
        implementor_fee: NanoErg,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        // Find how many nanoErgs are inside of the StableCoin boxes
        let sc_boxes_value = StableCoinBox::sum_nano_ergs_value(&sc_boxes);
        // Find how many StableCoins are inside of the StableCoin boxes
        let sc_boxes_total_sc = StableCoinBox::sum_token_amount(sc_boxes);

        // The amount of nanoErgs in the rc_boxes + the value of the
        // ReserveCoins being redeemed - the transaction fee
        let rb_value =
            sc_boxes_value + stablecoin_value_in_base - implementor_fee - transaction_fee;

        // Specify the registers in the Receipt box
        let rb_registers_vec = vec![
            (0 - amount_to_redeem as i64).into(),
            (0 - stablecoin_value_in_base as i64).into(),
        ];

        // Define the tokens
        let mut rb_tokens = vec![];
        // Check if there are any extra tokens that aren't being redeemed
        // and include them in the output
        if sc_boxes_total_sc > amount_to_redeem {
            // Define the StableCoin token
            let amount = sc_boxes_total_sc - amount_to_redeem;
            let new_sc_token = new_stablecoin_token(amount, bank_box)?;
            rb_tokens.push(new_sc_token)
        }
        // Find all other tokens held in user-provided input boxes
        let mut other_tokens =
            find_and_sum_other_tokens(&vec![bank_box.tokens()[0].clone()], &no_bank_inputs);
        rb_tokens.append(&mut other_tokens);

        let candidate = create_candidate(
            rb_value,
            &user_address,
            &rb_tokens,
            &rb_registers_vec,
            current_height,
        )?;
        Ok(candidate)
    }
}

// Creates a new StableCoin token with a custom amount
fn new_stablecoin_token(amount: u64, bank_box: &BankBox) -> Result<Token, ProtocolError> {
    let bank_stablecoin_token = bank_box.get_box().tokens[0].clone();
    let token_amount = TokenAmount::try_from(amount)
        .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
    let stablecoin_token = Token {
        token_id: bank_stablecoin_token.token_id.clone(),
        amount: token_amount,
    };
    Ok(stablecoin_token)
}

// Creates a new ReserveCoin token with a custom amount
fn new_reservecoin_token(amount: u64, bank_box: &BankBox) -> Result<Token, ProtocolError> {
    let bank_reservecoin_token = bank_box.get_box().tokens[1].clone();
    let token_amount = TokenAmount::try_from(amount)
        .map_err(|_| ProtocolError::Other("Invalid Token Amount".to_string()))?;
    let stablecoin_token = Token {
        token_id: bank_reservecoin_token.token_id.clone(),
        amount: token_amount,
    };
    Ok(stablecoin_token)
}
