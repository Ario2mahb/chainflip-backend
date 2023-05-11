use crate::eth::{EthereumCall, Tokenizable};
use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::{Address, Token};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::{vec, vec::Vec};

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct SetGovKeyWithAggKey {
	/// The new gov key.
	pub new_gov_key: Address,
}

impl SetGovKeyWithAggKey {
	pub fn new(new_gov_key: Address) -> Self {
		Self { new_gov_key }
	}
}

impl EthereumCall for SetGovKeyWithAggKey {
	const FUNCTION_NAME: &'static str = "setGovKeyWithAggKey";

	fn function_params() -> Vec<(&'static str, ethabi::ParamType)> {
		vec![("newGovKey", Address::param_type())]
	}

	fn function_call_args(&self) -> Vec<Token> {
		vec![self.new_gov_key.tokenize()]
	}
}

#[cfg(test)]
mod test_set_gov_key_with_agg_key {

	use super::*;
	use crate::{
		eth::{
			api::abi::load_abi, tests::asymmetrise, EthereumTransactionBuilder,
			SchnorrVerificationComponents,
		},
		ApiCall,
	};
	use ethabi::Token;
	use ethereum_types::H160;

	use crate::eth::api::{
		set_gov_key_with_agg_key::SetGovKeyWithAggKey, EthereumReplayProtection,
	};

	#[test]
	fn test_known_payload() {
		const FAKE_NONCE_TIMES_G_ADDR: [u8; 20] = asymmetrise([0x7f; 20]);
		const FAKE_SIG: [u8; 32] = asymmetrise([0xe1; 32]);
		const FAKE_KEYMAN_ADDR: [u8; 20] = asymmetrise([0xcf; 20]);
		const CHAIN_ID: u64 = 1;
		const NONCE: u64 = 6;
		const TEST_ADDR: [u8; 20] = asymmetrise([0xcf; 20]);

		let key_manager = load_abi("IKeyManager");

		let tx_builder = EthereumTransactionBuilder::new_unsigned(
			EthereumReplayProtection {
				nonce: NONCE,
				chain_id: CHAIN_ID,
				key_manager_address: FAKE_KEYMAN_ADDR.into(),
				contract_address: FAKE_KEYMAN_ADDR.into(),
			},
			SetGovKeyWithAggKey::new(H160::from(TEST_ADDR)),
		);

		assert_eq!(
			// Our encoding:
			tx_builder
				.signed(&SchnorrVerificationComponents {
					s: FAKE_SIG,
					k_times_g_address: FAKE_NONCE_TIMES_G_ADDR,
				})
				.chain_encoded(),
			// "Canonical" encoding based on the abi definition above and using the ethabi crate:
			key_manager
				.function("setGovKeyWithAggKey")
				.unwrap()
				.encode_input(&[
					// sigData: SigData(uint, uint, address)
					Token::Tuple(vec![
						Token::Uint(FAKE_SIG.into()),
						Token::Uint(NONCE.into()),
						Token::Address(FAKE_NONCE_TIMES_G_ADDR.into()),
					]),
					Token::Address(TEST_ADDR.into()),
				])
				.unwrap()
		);
	}
}
