#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub use inherent_provider::*;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sidechain_domain::*;
use sp_inherents::*;
use sp_runtime::{scale_info::TypeInfo, traits::Block as BlockT};

#[cfg(test)]
mod tests;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"nattoken";

#[derive(Default, Debug, Clone, PartialEq, Eq, TypeInfo, Encode, Decode, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MainChainScripts {
	pub native_token_policy: PolicyId,
	pub native_token_asset_name: AssetName,
	pub illiquid_supply_address: MainchainAddress,
}

sp_api::decl_runtime_apis! {
	pub trait NativeTokenManagementApi {
		fn get_main_chain_scripts() -> MainChainScripts;
	}
}

#[derive(Decode, Encode)]
pub struct TokenTransferData {
	pub token_amount: NativeTokenAmount,
}

#[derive(Encode, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, thiserror::Error))]
pub enum InherentError {
	#[cfg_attr(feature = "std", error("Inherent missing for token transfer of {}", 0.0))]
	TokenTransferNotHandled(NativeTokenAmount),
	#[cfg_attr(
		feature = "std",
		error("Incorrect token transfer amount: expected {}, got {}", 0.0, 1.0)
	)]
	IncorrectTokenNumberTransfered(NativeTokenAmount, NativeTokenAmount),
	#[cfg_attr(feature = "std", error("Unexpected transfer of {} tokens", 0.0))]
	UnexpectedTokenTransferInherent(NativeTokenAmount),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		true
	}
}

#[cfg(feature = "std")]
mod inherent_provider {
	use super::*;
	use sidechain_mc_hash::get_mc_hash_for_block;
	use sp_api::{ApiError, ProvideRuntimeApi};
	use sp_blockchain::HeaderBackend;
	use std::error::Error;
	use std::sync::Arc;

	#[async_trait::async_trait]
	pub trait NativeTokenManagementDataSource {
		type Error;

		/// Retrieves total of native token transfers into the illiquid supply in the range (after_block, to_block]
		async fn get_total_native_token_transfer(
			&self,
			after_block: Option<McBlockHash>,
			to_block: McBlockHash,
			native_token_policy_id: PolicyId,
			native_token_asset_name: AssetName,
			illiquid_supply_address: MainchainAddress,
		) -> Result<NativeTokenAmount, Self::Error>;
	}

	pub struct NativeTokenManagementInherentDataProvider {
		pub token_amount: NativeTokenAmount,
	}

	#[derive(thiserror::Error, sp_runtime::RuntimeDebug)]
	pub enum IDPCreationError {
		#[error("Failed to read native token data from data source: {0:?}")]
		DataSourceError(Box<dyn Error + Send + Sync>),
		#[error("Failed to retrieve main chain scripts from the runtime: {0:?}")]
		GetMainChainScriptsError(ApiError),
		#[error("Failed to retrieve previous MC hash: {0:?}")]
		McHashError(Box<dyn Error + Send + Sync>),
	}

	impl NativeTokenManagementInherentDataProvider {
		pub async fn new<Block, C, E>(
			client: Arc<C>,
			data_source: &(dyn NativeTokenManagementDataSource<Error = E> + Send + Sync),
			mc_hash: McBlockHash,
			parent_hash: <Block as BlockT>::Hash,
		) -> Result<Self, IDPCreationError>
		where
			Block: BlockT,
			C: HeaderBackend<Block>,
			C: ProvideRuntimeApi<Block> + Send + Sync,
			C::Api: NativeTokenManagementApi<Block>,
			E: std::error::Error + Send + Sync + 'static,
		{
			let api = client.runtime_api();
			let scripts = api
				.get_main_chain_scripts(parent_hash)
				.map_err(IDPCreationError::GetMainChainScriptsError)?;
			let parent_mc_hash: Option<McBlockHash> =
				get_mc_hash_for_block(client.as_ref(), parent_hash)
					.map_err(IDPCreationError::McHashError)?;
			let token_amount = data_source
				.get_total_native_token_transfer(
					parent_mc_hash,
					mc_hash,
					scripts.native_token_policy,
					scripts.native_token_asset_name,
					scripts.illiquid_supply_address,
				)
				.await
				.map_err(|err| IDPCreationError::DataSourceError(Box::new(err)))?;

			Ok(Self { token_amount })
		}
	}

	#[async_trait::async_trait]
	impl InherentDataProvider for NativeTokenManagementInherentDataProvider {
		async fn provide_inherent_data(
			&self,
			inherent_data: &mut InherentData,
		) -> Result<(), sp_inherents::Error> {
			inherent_data.put_data(
				INHERENT_IDENTIFIER,
				&TokenTransferData { token_amount: self.token_amount.clone() },
			)
		}

		async fn try_handle_error(
			&self,
			identifier: &InherentIdentifier,
			mut error: &[u8],
		) -> Option<Result<(), sp_inherents::Error>> {
			if *identifier != INHERENT_IDENTIFIER {
				return None;
			}

			let error = InherentError::decode(&mut error).ok()?;

			Some(Err(sp_inherents::Error::Application(Box::from(error))))
		}
	}

	#[cfg(any(test, feature = "mock"))]
	pub mod mock {
		use crate::NativeTokenManagementDataSource;
		use async_trait::async_trait;
		use core::marker::PhantomData;
		use derive_new::new;
		use sidechain_domain::*;
		use std::collections::HashMap;

		#[derive(new, Default)]
		pub struct MockNativeTokenDataSource<Err> {
			transfers: HashMap<(Option<McBlockHash>, McBlockHash), NativeTokenAmount>,
			_marker: PhantomData<Err>,
		}

		#[async_trait]
		impl<Err> NativeTokenManagementDataSource for MockNativeTokenDataSource<Err>
		where
			Err: std::error::Error + Send + Sync,
		{
			type Error = Err;

			async fn get_total_native_token_transfer(
				&self,
				after_block: Option<McBlockHash>,
				to_block: McBlockHash,
				_native_token_policy_id: PolicyId,
				_native_token_asset_name: AssetName,
				_illiquid_supply_address: MainchainAddress,
			) -> Result<NativeTokenAmount, Self::Error> {
				Ok(self.transfers.get(&(after_block, to_block)).cloned().unwrap_or_default())
			}
		}
	}
}
