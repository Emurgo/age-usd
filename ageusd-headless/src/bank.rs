// This file specifies the Bank stage (aka. the BankBox).
// The BankBox implements methods for acquiring it via a `BoxSpec` interface,
// methods for reading current state of the protocol, as well as
// methods for building an output box (creating `ErgoBoxCandidate`s for
// Actions within the protocol).
use crate::equations::reserve_ratio;
use crate::error::ProtocolError;
use crate::parameters::{
    BANK_NFT_ID, COOLING_OFF_HEIGHT, FEE_PERCENT, IMPLEMENTOR_FEE_PERCENT, MAX_RESERVE_RATIO,
    MIN_BOX_VALUE, MIN_RESERVE_RATIO, RESERVECOIN_DEFAULT_PRICE, RESERVECOIN_TOKEN_ID,
    STABLECOIN_TOKEN_ID,
};
use ergo_headless_dapp_framework::encoding::{build_token, unwrap_long};
use ergo_headless_dapp_framework::{
    create_candidate, BoxSpec, ErgUsdOraclePoolBox, ExplorerFindable, HeadlessDappError, SpecBox,
    SpecifiedBox, TokenSpec, WASMBox, WrapBox, WrappedBox,
};
use ergo_headless_dapp_framework::{BlockHeight, NanoErg, P2SAddressString};
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::transaction::TxId;
use ergo_lib_wasm::ergo_box::ErgoBox as WErgoBox;
use wasm_bindgen::prelude::*;

/// The struct which represents the `Bank` stage.
#[wasm_bindgen]
#[derive(Debug, Clone, WrapBox, SpecBox, WASMBox)]
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
        if self.base_reserves() <= self.liabilities(oracle_box) {
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
        if self.num_circulating_reservecoins() <= 1 || self.equity(oracle_box) == 0 {
            return RESERVECOIN_DEFAULT_PRICE;
        }
        self.equity(oracle_box) / self.num_circulating_reservecoins()
    }
    /// Number of StableCoins possible to be minted based off of current Reserve Ratio
    #[wasm_bindgen]
    pub fn num_able_to_mint_stablecoin_naive(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        self.equity(oracle_box) / oracle_box.datapoint_in_cents() / 4
    }

    /// Number of StableCoins possible to be minted based off of current Reserve Ratio
    #[wasm_bindgen]
    pub fn num_able_to_mint_stablecoin(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        // Start at approximately the right amount
        let mut num_to_mint = self.equity(oracle_box) / oracle_box.datapoint_in_cents() / 4;

        // Add self-adjusting increment to increase efficiency of function
        let mut increment_amount = 1;
        if self.num_circulating_stablecoins() > 100 {
            increment_amount = 10;
        }
        if self.num_circulating_stablecoins() > 1000000 {
            increment_amount = self.num_circulating_reservecoins() / 10000;
        }

        loop {
            let new_reserve_ratio = self.mint_stablecoin_reserve_ratio(oracle_box, num_to_mint);
            // If New Reserve Ratio is below minimum, meaning cannot mint anymore, then calculate final amount to mint and break
            if new_reserve_ratio <= MIN_RESERVE_RATIO {
                if (increment_amount + 1) >= num_to_mint {
                    break;
                }
                num_to_mint -= increment_amount + 1;
                loop {
                    let new_reserve_ratio =
                        self.mint_stablecoin_reserve_ratio(oracle_box, num_to_mint);
                    if new_reserve_ratio <= MIN_RESERVE_RATIO {
                        num_to_mint -= 1;
                        if num_to_mint == 0 {
                            return 0;
                        }
                        break;
                    }
                    num_to_mint += 1;
                }
                break;
            }
            // If still above Minimum Reserve Ratio, increase the `num_to_mint` and test again
            num_to_mint += increment_amount;
        }

        num_to_mint
    }

    /// Acquire the new reserve ratio after minting `num_to_mint` Stablecoins
    fn mint_stablecoin_reserve_ratio(
        &self,
        oracle_box: &ErgUsdOraclePoolBox,
        num_to_mint: u64,
    ) -> u64 {
        let new_base_reserves =
            self.base_reserves() + self.base_cost_to_mint_stablecoin(num_to_mint, oracle_box);
        reserve_ratio(
            new_base_reserves,
            self.num_circulating_stablecoins() + num_to_mint,
            oracle_box.datapoint_in_cents(),
        )
    }

    /// Number of ReserveCoins possible to be minted based off of current Reserve Ratio
    #[wasm_bindgen]
    pub fn num_able_to_mint_reservecoin(
        &self,
        oracle_box: &ErgUsdOraclePoolBox,
        current_height: BlockHeight,
    ) -> u64 {
        if current_height < COOLING_OFF_HEIGHT {
            return u64::MAX;
        }

        let mut num_to_mint = 0;

        // Add self-adjusting increment to increase efficiency of function
        let mut increment_amount = 1;
        if self.num_circulating_reservecoins() > 1000 {
            increment_amount = self.num_circulating_reservecoins() / 100;
        }

        loop {
            let new_reserve_ratio = self.mint_reservecoin_reserve_ratio(oracle_box, num_to_mint);
            // If New Reserve Ratio is below minimum, meaning cannot mint anymore, then calculate final amount to mint and break
            if new_reserve_ratio >= MAX_RESERVE_RATIO {
                if (increment_amount + 1) >= num_to_mint {
                    break;
                }
                num_to_mint -= increment_amount + 1;
                loop {
                    let new_reserve_ratio =
                        self.mint_reservecoin_reserve_ratio(oracle_box, num_to_mint);
                    if new_reserve_ratio >= MAX_RESERVE_RATIO {
                        num_to_mint -= 1;
                        if num_to_mint == 0 {
                            return 0;
                        }
                        break;
                    }
                    num_to_mint += 1;
                }
                break;
            }
            // If still above Minimum Reserve Ratio, increase the `num_to_mint` and test again
            num_to_mint += increment_amount;
        }

        num_to_mint
    }

    /// Acquire the new reserve ratio after minting `num_to_mint` Reservecoins
    fn mint_reservecoin_reserve_ratio(
        &self,
        oracle_box: &ErgUsdOraclePoolBox,
        num_to_mint: u64,
    ) -> u64 {
        let new_base_reserves =
            self.base_reserves() + self.base_cost_to_mint_reservecoin(num_to_mint, oracle_box);
        reserve_ratio(
            new_base_reserves,
            self.num_circulating_stablecoins(),
            oracle_box.datapoint_in_cents(),
        )
    }

    #[wasm_bindgen]
    pub fn num_able_to_redeem_reservecoin_naive(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        self.equity(oracle_box) / self.reservecoin_nominal_price(oracle_box)
    }

    /// Number of ReserveCoins possible to be redeemed based off of current Reserve Ratio.
    /// Checks if the provided `current_height` is before the COOLING_OFF_HEIGHT to verify
    /// as well.
    #[wasm_bindgen]
    pub fn num_able_to_redeem_reservecoin(&self, oracle_box: &ErgUsdOraclePoolBox) -> u64 {
        let mut num_to_redeem =
            self.equity(oracle_box) / self.reservecoin_nominal_price(oracle_box);

        // Add self-adjusting increment to increase efficiency of function
        let mut increment_amount = 1;
        if self.num_circulating_reservecoins() > 1000 {
            increment_amount = self.num_circulating_reservecoins() / 100;
        }
        if self.num_circulating_reservecoins() > 100000 {
            increment_amount = self.num_circulating_reservecoins() / 1000;
        }
        if self.num_circulating_reservecoins() > 10000000 {
            increment_amount = self.num_circulating_reservecoins() / 10000;
        }

        loop {
            let new_reserve_ratio =
                self.redeem_reservecoin_reserve_ratio(oracle_box, num_to_redeem);
            // If New Reserve Ratio is below minimum, meaning cannot mint anymore, then calculate final amount to mint and break
            if new_reserve_ratio <= MIN_RESERVE_RATIO {
                if (increment_amount + 1) >= num_to_redeem {
                    break;
                }
                num_to_redeem -= increment_amount + 1;
                loop {
                    let new_reserve_ratio =
                        self.redeem_reservecoin_reserve_ratio(oracle_box, num_to_redeem);
                    if new_reserve_ratio <= MIN_RESERVE_RATIO {
                        num_to_redeem -= 1;
                        if num_to_redeem == 0 {
                            return 0;
                        }
                        break;
                    }
                    num_to_redeem += 1;
                }
                break;
            }
            // If still above Minimum Reserve Ratio, increase the `num_to_redeem` and test again
            num_to_redeem += increment_amount;
        }

        num_to_redeem
    }

    /// Acquire the new reserve ratio after minting `num_to_redeem` Reservecoins
    fn redeem_reservecoin_reserve_ratio(
        &self,
        oracle_box: &ErgUsdOraclePoolBox,
        num_to_redeem: u64,
    ) -> u64 {
        let new_base_reserves =
            self.base_reserves() - self.base_cost_to_mint_reservecoin(num_to_redeem, oracle_box);
        reserve_ratio(
            new_base_reserves,
            self.num_circulating_stablecoins(),
            oracle_box.datapoint_in_cents(),
        )
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

    /// The amount of nanoErg fees for minting StableCoins.
    /// This includes protocol fees, tx fees, and implementor fees.
    #[wasm_bindgen]
    pub fn fees_from_minting_stablecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let feeless_amount = self.stablecoin_nominal_price(oracle_box) * amount_to_mint;
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;
        let implementor_fee = (self.base_cost_to_mint_stablecoin(amount_to_mint, oracle_box) as f64
            * IMPLEMENTOR_FEE_PERCENT) as u64;
        protocol_fee + transaction_fee + implementor_fee
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
    #[wasm_bindgen]
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

    /// The amount of nanoErg fees for minting ReserveCoins.
    /// This includes protocol fees, tx fees, and implementor fees.
    #[wasm_bindgen]
    pub fn fees_from_minting_reservecoin(
        &self,
        amount_to_mint: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let feeless_amount = self.reservecoin_nominal_price(oracle_box) * amount_to_mint;
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;
        let implementor_fee = (self.base_cost_to_mint_reservecoin(amount_to_mint, oracle_box)
            as f64
            * IMPLEMENTOR_FEE_PERCENT) as u64;
        protocol_fee + transaction_fee + implementor_fee
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
        let base_amount = self.base_amount_from_redeeming_reservecoin(amount_to_redeem, oracle_box);
        let fees = transaction_fee + (base_amount as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;

        if base_amount > fees {
            return base_amount - fees;
        } else {
            return 0;
        }
    }

    /// The amount of nanoErg fees for redeeming ReserveCoins.
    /// This includes protocol fees, tx fees, and implementor fees.
    #[wasm_bindgen]
    pub fn fees_from_redeeming_reservecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let feeless_amount = self.reservecoin_nominal_price(oracle_box) * amount_to_redeem;
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;
        let implementor_fee =
            (self.base_amount_from_redeeming_reservecoin(amount_to_redeem, oracle_box) as f64
                * IMPLEMENTOR_FEE_PERCENT) as u64;
        protocol_fee + transaction_fee + implementor_fee
    }

    /// The amount of base currency (Ergs) which will be redeemed
    /// from the protocol based on current reserves + the number of
    /// ReserveCoins being redeemed by the user. Includes protocol fee.
    #[wasm_bindgen]
    pub fn base_amount_from_redeeming_reservecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
    ) -> u64 {
        // Cost to mint without fees
        let feeless_amount = self.reservecoin_nominal_price(oracle_box) * amount_to_redeem;
        // The StableCoin protocol fee charged
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;

        feeless_amount - protocol_fee
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
        let base_amount = self.base_amount_from_redeeming_stablecoin(amount_to_redeem, oracle_box);
        let fees = transaction_fee + (base_amount as f64 * IMPLEMENTOR_FEE_PERCENT) as u64;

        if base_amount > fees {
            return base_amount - fees;
        } else {
            return 0;
        }
    }

    /// The amount of nanoErg fees for redeeming StableCoins.
    /// This includes protocol fees, tx fees, and implementor fees.
    #[wasm_bindgen]
    pub fn fees_from_redeeming_stablecoin(
        &self,
        amount_to_redeem: u64,
        oracle_box: &ErgUsdOraclePoolBox,
        transaction_fee: u64,
    ) -> u64 {
        let feeless_amount = self.stablecoin_nominal_price(oracle_box) * amount_to_redeem;
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;
        let implementor_fee =
            (self.base_amount_from_redeeming_stablecoin(amount_to_redeem, oracle_box) as f64
                * IMPLEMENTOR_FEE_PERCENT) as u64;
        protocol_fee + transaction_fee + implementor_fee
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
        let feeless_amount = self.stablecoin_nominal_price(oracle_box) * amount_to_redeem;
        let protocol_fee = feeless_amount * FEE_PERCENT / 100;

        feeless_amount - protocol_fee
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
        let reservecoin_tokens = build_token(
            RESERVECOIN_TOKEN_ID,
            u64::from(bank_reservecoin_token_in.amount) - amount_to_mint,
        )?;
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
        let stablecoin_tokens = build_token(
            STABLECOIN_TOKEN_ID,
            u64::from(bank_stablecoin_token_in.amount) - amount_to_mint,
        )?;
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
        let reservecoin_tokens = build_token(
            RESERVECOIN_TOKEN_ID,
            u64::from(bank_reservecoin_token_in.amount) + amount_to_redeem,
        )?;
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
        let stablecoin_tokens = build_token(
            STABLECOIN_TOKEN_ID,
            u64::from(bank_stablecoin_token_in.amount) + amount_to_redeem,
        )?;
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
