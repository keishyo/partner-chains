use parity_scale_codec::{Decode, Encode};
use plutus::*;
use scale_info::TypeInfo;
use sidechain_domain::*;

/// The part of data for selection of authorities that comes from the main chain.
/// It is unfiltered, so the selection algorithm should filter out invalid candidates.
#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct AuthoritySelectionInputs {
	pub d_parameter: DParameter,
	pub permissioned_candidates: Vec<PermissionedCandidateData>,
	pub registered_candidates: Vec<CandidateRegistrations>,
	pub epoch_nonce: EpochNonce,
}

// #[derive(Debug, PartialEq, Eq, Clone, Decode, thiserror::Error, Serialize, Deserialize)]
#[cfg(feature = "std")]
#[derive(Debug, thiserror::Error)]
pub enum AuthoritySelectionInputsCreationError {
	#[cfg_attr(feature = "std", error("Failed to get Ariadne parameters for epoch: {0}, D-parameter: {1:?}, permissioned candidates: {2:?}: {3}"))]
	AriadneParametersQuery(
		McEpochNumber,
		PolicyId,
		PolicyId,
		Box<dyn std::error::Error + Send + Sync>,
	),
	#[cfg_attr(feature = "std", error("Failed to get registered candidates for epoch: {0}, committee candidate address: {1}: {2}."))]
	GetCandidatesQuery(McEpochNumber, String, Box<dyn std::error::Error + Send + Sync>),
	#[cfg_attr(feature = "std", error("Failed to get epoch nonce for epoch: {0}: {1}."))]
	GetEpochNonceQuery(McEpochNumber, Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq)]
pub struct AriadneParameters {
	pub d_parameter: DParameter,
	pub permissioned_candidates: Vec<PermissionedCandidateData>,
}

#[cfg(feature = "std")]
/// Queries about the Authority Candidates
#[async_trait::async_trait]
pub trait AuthoritySelectionDataSource {
	type Error;

	/// Returns D-parameter and list of permissioned candidates that is effective for the given epoch.
	/// The data from the latest block of `data_epoch(epoch)` will be used if available, otherwise returns data at the latest block of the chain.
	async fn get_ariadne_parameters(
		&self,
		epoch_number: McEpochNumber,
		d_parameter: PolicyId,
		permissioned_candidates: PolicyId,
	) -> Result<AriadneParameters, Self::Error>;

	/// Returns the list of registrations that is effective for the given epoch.
	/// The data from the latest block of `data_epoch(epoch)` will be used if available, otherwise returns data at the latest block of the chain.
	/// Each item is a list of one candidate registrations.
	async fn get_candidates(
		&self,
		epoch: McEpochNumber,
		committee_candidate_address: MainchainAddress,
	) -> Result<Vec<CandidateRegistrations>, Self::Error>;

	/// Returns Cardano Epoch Nonce. None, if the nonce for given epoch is not known yet.
	async fn get_epoch_nonce(
		&self,
		epoch: McEpochNumber,
	) -> Result<Option<EpochNonce>, Self::Error>;

	///
	/// # Arguments
	///
	/// * `for_epoch`: main chain epoch number during which candidate data is meant to be used
	///
	/// returns: Result<McEpochNumber, DataSourceError> - data source methods called with `for_epoch` will query only for data which was stored on main chain in the returned epoch or earlier
	///
	///
	async fn data_epoch(&self, for_epoch: McEpochNumber) -> Result<McEpochNumber, Self::Error>;
}

impl AuthoritySelectionInputs {
	#[cfg(feature = "std")]
	pub async fn from_mc_data<E>(
		candidate_data_source: &(dyn AuthoritySelectionDataSource<Error = E> + Send + Sync),
		for_epoch: McEpochNumber,
		scripts: sp_session_validator_management::MainChainScripts,
	) -> Result<Self, AuthoritySelectionInputsCreationError>
	where
		E: std::error::Error + Send + Sync + 'static,
	{
		let ariadne_parameters_response = candidate_data_source
			.get_ariadne_parameters(
				for_epoch,
				scripts.d_parameter_policy.clone(),
				scripts.permissioned_candidates_policy.clone(),
			)
			.await
			.map_err(|err| {
				AuthoritySelectionInputsCreationError::AriadneParametersQuery(
					for_epoch,
					scripts.d_parameter_policy,
					scripts.permissioned_candidates_policy,
					Box::new(err),
				)
			})?;

		let d_parameter = ariadne_parameters_response.d_parameter;
		let permissioned_candidates = ariadne_parameters_response.permissioned_candidates;

		let registered_candidates: Vec<CandidateRegistrations> = candidate_data_source
			.get_candidates(for_epoch, scripts.committee_candidate_address.clone())
			.await
			.map_err(|err| {
				AuthoritySelectionInputsCreationError::GetCandidatesQuery(
					for_epoch,
					scripts.committee_candidate_address.to_string(),
					Box::new(err),
				)
			})?;
		let epoch_nonce_response =
			candidate_data_source.get_epoch_nonce(for_epoch).await.map_err(|err| {
				AuthoritySelectionInputsCreationError::GetEpochNonceQuery(for_epoch, Box::new(err))
			})?;
		let epoch_nonce = epoch_nonce_response.unwrap_or(EpochNonce(vec![]));

		Ok(Self { d_parameter, permissioned_candidates, registered_candidates, epoch_nonce })
	}
}
