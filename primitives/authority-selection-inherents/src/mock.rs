use crate::authority_selection_inputs::*;
use core::marker::PhantomData;
use sidechain_domain::*;

#[derive(Clone, Default)]
pub struct MockAuthoritySelectionDataSource<Err> {
	/// Each entry in each field is returned when queried for epoch equal to its index.
	/// For example `candidates[0]` is the list of candidates that will be returned for epoch 0.
	/// `candidates[1]` is the list of candidates that will be returned for epoch 1 and so on.
	pub candidates: Vec<Vec<CandidateRegistrations>>,
	pub permissioned_candidates: Vec<Option<Vec<PermissionedCandidateData>>>,
	pub _marker: PhantomData<Err>,
}

#[async_trait::async_trait]
impl<Err> AuthoritySelectionDataSource for MockAuthoritySelectionDataSource<Err>
where
	Err: std::error::Error + Send + Sync + 'static,
{
	type Error = Err;

	async fn get_ariadne_parameters(
		&self,
		epoch_number: McEpochNumber,
		_d_parameter: PolicyId,
		_permissioned_candidates: PolicyId,
	) -> Result<AriadneParameters, Self::Error> {
		match self.permissioned_candidates.get(epoch_number.0 as usize) {
			Some(Some(candidates)) => Ok(AriadneParameters {
				d_parameter: DParameter {
					num_permissioned_candidates: 3,
					num_registered_candidates: 2,
				},
				permissioned_candidates: candidates.clone(),
			}),
			_ => panic!("mock was called with unexpected argument"),
		}
	}

	async fn get_candidates(
		&self,
		epoch_number: McEpochNumber,
		_committee_candidate_address: MainchainAddress,
	) -> Result<Vec<CandidateRegistrations>, Self::Error> {
		Ok(self.candidates.get(epoch_number.0 as usize).cloned().unwrap_or(vec![]))
	}

	async fn get_epoch_nonce(
		&self,
		_epoch: McEpochNumber,
	) -> Result<Option<EpochNonce>, Self::Error> {
		Ok(Some(EpochNonce(vec![42u8])))
	}

	async fn data_epoch(&self, for_epoch: McEpochNumber) -> Result<McEpochNumber, Self::Error> {
		Ok(for_epoch)
	}
}
