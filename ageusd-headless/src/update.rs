// This file holds logic for Actions related to updating the StableCoin
// protocol and the definitions of boxes related to voting/updating.
use crate::bank::BankBox;
use crate::error::ProtocolError;
use crate::input_boxes::*;
use crate::parameters::{UPDATE_BALLOT_TOKEN_ID, UPDATE_NFT_ID};
use crate::protocol::StableCoinProtocol;
use ergo_headless_dapp_framework::encoding::{
    hash_and_serialize_p2s, serialize_hex_encoded_string, serialize_p2s_from_ergo_tree,
    unwrap_hex_encoded_string,
};
use ergo_headless_dapp_framework::{
    create_candidate, HeadlessDappError, TokensChangeBox, TxFeeBox, WrapBox, WrappedBox,
};
use ergo_headless_dapp_framework::{BlockHeight, NanoErg, P2PKAddressString, P2SAddressString};
pub use ergo_headless_dapp_framework::{
    BoxSpec, ExplorerFindable, SpecBox, SpecifiedBox, TokenSpec,
};
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::transaction::unsigned::UnsignedTransaction;
use ergo_lib::chain::transaction::UnsignedInput;

/// A box which represents a cast vote for updating the protocol
#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct BallotBox {
    ergo_box: ErgoBox,
}

impl SpecifiedBox for BallotBox {
    /// A `BoxSpec` that checks that the box is a valid BallotBox
    fn box_spec() -> BoxSpec {
        let tok_1_spec = Some(TokenSpec::new(1..u64::MAX, UPDATE_BALLOT_TOKEN_ID));
        let tok_specs = vec![tok_1_spec];
        // Add registers specs once implemented (just on type)
        BoxSpec::new_predicated(None, None, vec![], tok_specs, Some(Self::predicate))
    }
}

// Methods for acquiring the state of the BallotBox
impl BallotBox {
    // The hash of the address which is being voted for in the Ballot Box
    pub fn address_hash_voted_for(&self) -> String {
        unwrap_hex_encoded_string(&self.registers()[0]).unwrap()
    }

    // The box id of the Update Box when the vote was cast
    pub fn update_box_id(&self) -> String {
        unwrap_hex_encoded_string(&self.registers()[1]).unwrap()
    }

    pub fn voting_power(&self) -> u64 {
        self.tokens()[0].amount.into()
    }
}

impl BallotBox {
    /// A simple predicate to check the registers
    pub fn predicate(b: &ErgoBox) -> bool {
        b.additional_registers.get_ordered_values().len() == 2
    }

    /// Verify that the `BallotBox` is voting for a given P2S address
    pub fn is_voting_for_address(&self, address: &P2PKAddressString) -> bool {
        if let Ok(address_hash_constant) = hash_and_serialize_p2s(address) {
            let address_hash = unwrap_hex_encoded_string(&address_hash_constant).unwrap();
            if address_hash == self.address_hash_voted_for() {
                return true;
            }
        }
        false
    }

    /// Create an `ErgoBoxCandidate` for the output Ballot box for the
    /// `Vote For Update` action
    pub fn create_vote_for_update_candidate(
        &self,
        address_to_vote_for: &P2SAddressString,
        update_box: &UpdateBox,
        user_input_ballot_box: &BallotBox,
        current_height: BlockHeight,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let input_box = user_input_ballot_box.get_box();

        // Get the `Constant` for the blake2b_256 hash of the address' ErgoTree
        if let Ok(address_hash_constant) = hash_and_serialize_p2s(address_to_vote_for) {
            // Get the `Constant` for the Update Box id
            if let Ok(box_id_constant) =
                serialize_hex_encoded_string(&update_box.get_box().box_id().into())
            {
                // Set R4 to the address hash constant
                // Set R5 to the box id constant
                let registers_vec = vec![address_hash_constant, box_id_constant];

                // Creating the output Ballot box candidate
                let output_ballot_candidate = create_candidate(
                    input_box.value.as_u64().clone(),
                    &serialize_p2s_from_ergo_tree(input_box.ergo_tree),
                    &input_box.tokens,
                    &registers_vec,
                    current_height,
                )?;

                return Ok(output_ballot_candidate);
            }
        }
        Err(ProtocolError::Other("Failed to create `Vote For Update` Action Tx due to failed serialization of registers.".to_string()))
    }
}

/// The box which holds the Update NFT & the address to be used to
/// update the protocol in R4.
#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct UpdateBox {
    ergo_box: ErgoBox,
}

impl SpecifiedBox for UpdateBox {
    /// A `BoxSpec` that checks that the box is a valid UpdateBox
    fn box_spec() -> BoxSpec {
        let tok_1_spec = Some(TokenSpec::new(1..2, UPDATE_NFT_ID));
        let tok_specs = vec![tok_1_spec];
        // Add registers specs once implemented (just on type)
        BoxSpec::new(None, None, vec![], tok_specs)
    }
}

impl UpdateBox {
    /// Create an `ErgoBoxCandidate` for the output Update box for the
    /// `Collect Votes` Action
    pub fn create_collect_votes_candidate(
        &self,
        address_voted_for: &P2SAddressString,
        update_input_box: &UpdateBox,
        current_height: BlockHeight,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let input_box = update_input_box.get_box();

        // Get the address_hash_constant
        if let Ok(address_hash_constant) = hash_and_serialize_p2s(address_voted_for) {
            // Set R4 to the address hash constant
            let registers_vec = vec![address_hash_constant];

            // Creating the output Ballot box candidate
            let output_ballot_candidate = create_candidate(
                input_box.value.as_u64().clone(),
                &update_input_box.p2s_address(),
                &input_box.tokens,
                &registers_vec,
                current_height,
            )?;

            return Ok(output_ballot_candidate);
        }
        Err(ProtocolError::Other("Failed to create `Vote For Update` Action Tx due to failed serialization of registers.".to_string()))
    }

    /// Create an `ErgoBoxCandidate` for the output Update box for the
    /// `Update Protocol` Action
    pub fn create_update_protocol_candidate(
        &self,
        update_input_box: &UpdateBox,
        current_height: BlockHeight,
    ) -> Result<ErgoBoxCandidate, ProtocolError> {
        let input_box = update_input_box.get_box();

        let candidate = create_candidate(
            update_input_box.nano_ergs(),
            &update_input_box.p2s_address(),
            &input_box.tokens,
            &update_input_box.registers(),
            current_height,
        )?;

        Ok(candidate)
    }
}

/// Implement Update Actions on the `StableCoinProtocol`
impl StableCoinProtocol {
    /// Action: Vote For Update
    pub fn action_vote_for_update(
        &self,
        address_to_vote_for: P2SAddressString,
        user_ballot_box: BallotBox,
        update_box: UpdateBox,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        ergs_box_for_fee: ErgsBox,
        user_address: P2PKAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        // Number of nanoErgs left over from the input after paying for fee
        let left_over_nano_ergs = ergs_box_for_fee.get_box().value.as_u64() - transaction_fee;

        // Defining inputs
        let tx_inputs = vec![
            user_ballot_box.get_box().into(),
            ergs_box_for_fee.get_box().into(),
        ];

        // Create the output Ballot Box candidate
        let output_ballot_candidate = user_ballot_box.create_vote_for_update_candidate(
            &address_to_vote_for,
            &update_box,
            &user_ballot_box,
            current_height,
        )?;

        let change_box_candidate = TokensChangeBox::output_candidate_filtered(
            &vec![user_ballot_box.tokens()[0].clone()],
            &vec![ergs_box_for_fee.get_box()],
            left_over_nano_ergs,
            &user_address,
            current_height,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Creating the UnsignedTransaction
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            vec![],
            vec![
                output_ballot_candidate,
                change_box_candidate,
                transaction_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }

    /// Action: Collect Votes
    pub fn action_collect_votes(
        &self,
        address_voted_for: &P2SAddressString,
        ballot_boxes: &Vec<BallotBox>,
        update_box: &UpdateBox,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        ergs_box_for_fee: &ErgsBox,
        user_address: &P2PKAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        // Number of nanoErgs left over from the input after paying for fee
        let left_over_nano_ergs = ergs_box_for_fee.get_box().value.as_u64() - transaction_fee;

        // Verify that the Collect Votes Action can be issued
        // by checking if the Update Box already has the same
        // value in R4.
        let current_update_box_r4 = update_box.registers()[0].clone();
        if let Ok(address_hash_constant) = hash_and_serialize_p2s(address_voted_for) {
            if address_hash_constant == current_update_box_r4 {
                return Err(ProtocolError::Other(
                    "The Update Box already has the same address collected in R4.".to_string(),
                ));
            }
        }

        // Defining inputs
        let tx_inputs: Vec<UnsignedInput> = vec![
            update_box.get_box().into(),
            ergs_box_for_fee.get_box().into(),
        ];

        // Filter out all `BallotBox`es which are voting for
        // a different update address or have a old Update Box id.
        let mut data_inputs = vec![];
        for bb in ballot_boxes {
            if bb.update_box_id() == update_box.box_id()
                && bb.is_voting_for_address(address_voted_for)
            {
                data_inputs.push(bb.as_data_input());
            }
        }

        // Create the output Update Box candidate
        let output_update_candidate = update_box.create_collect_votes_candidate(
            &address_voted_for,
            &update_box,
            current_height,
        )?;

        let change_box_candidate = TokensChangeBox::output_candidate(
            &vec![ergs_box_for_fee.get_box()],
            left_over_nano_ergs,
            &user_address.clone(),
            current_height,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Creating the UnsignedTransaction
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            data_inputs,
            vec![
                output_update_candidate,
                change_box_candidate,
                transaction_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }

    /// Action: Update Protocol
    pub fn action_update_protocol(
        &self,
        update_address: &P2SAddressString,
        update_box: &UpdateBox,
        bank_box: &BankBox,
        current_height: BlockHeight,
        transaction_fee: NanoErg,
        ergs_box_for_fee: &ErgsBox,
        user_address: &P2PKAddressString,
    ) -> Result<UnsignedTransaction, ProtocolError> {
        // Number of nanoErgs left over from the input after paying for fee
        let left_over_nano_ergs = ergs_box_for_fee.get_box().value.as_u64() - transaction_fee;

        // Defining inputs
        let tx_inputs: Vec<UnsignedInput> = vec![
            update_box.as_unsigned_input(),
            bank_box.as_unsigned_input(),
            ergs_box_for_fee.as_unsigned_input(),
        ];

        let bank_box_candidate =
            bank_box.create_update_protocol_candidate(update_address, bank_box, current_height)?;

        let update_box_candidate =
            update_box.create_update_protocol_candidate(update_box, current_height)?;

        let change_box_candidate = TokensChangeBox::output_candidate(
            &vec![ergs_box_for_fee.get_box()],
            left_over_nano_ergs,
            &user_address.clone(),
            current_height,
        )?;

        // Create the Transaction Fee box candidate
        let transaction_fee_box_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height)?;

        // Creating the UnsignedTransaction
        let unsigned_tx = UnsignedTransaction::new(
            tx_inputs,
            vec![],
            vec![
                update_box_candidate,
                bank_box_candidate,
                change_box_candidate,
                transaction_fee_box_candidate,
            ],
        );

        Ok(unsigned_tx)
    }
}
