#![cfg_attr(not(feature = "std"), no_std)]

pub mod info_types;
pub use info_types::{
	AgreementInfo, Participant, RemoteStorage, RemoteStorageProvider, ServiceAgreement, VoteInfo,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Todo: Implement Benchmarking for Pallet
// #[cfg(any(test, feature = "runtime-benchmarks"))]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		sp_std::{prelude::*, vec::Vec},
		traits::{Currency, ReservableCurrency, StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;

		/// How many signatures needed for the agreement to proceed to escrow.
		#[pallet::constant]
		type NumVotesForApproval: Get<u32>;

		/// maximum num of proposals for a single user
		#[pallet::constant]
		type MaxProposalsForUser: Get<u32>;

		type EscrowPallet;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type AgreementId<T> = <T as frame_system::Config>::Hash;

	/// Query Agreement which has been proposed but not approved
	#[pallet::storage]
	#[pallet::getter(fn proposed_service_agreements)]
	pub type ProposedServiceAgreements<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AgreementId<T>,
		ServiceAgreement<T::AccountId>,
		OptionQuery,
	>;

	/// Query the voting information for a particular proposal.
	#[pallet::storage]
	#[pallet::getter(fn proposal_votes)]
	pub type ProposalVotes<T: Config> =
		StorageMap<_, Blake2_128Concat, AgreementId<T>, VoteInfo<T::AccountId>, OptionQuery>;

	/// Query the proposals which this account has signed(voted) for.
	#[pallet::storage]
	#[pallet::getter(fn proposal_votes_for_account)]
	pub type ProposalVotesForAccount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<AgreementId<T>>, ValueQuery>;

	/// Query the number of proposals for a particular account
	#[pallet::storage]
	#[pallet::getter(fn num_proposals_for_account)]
	pub type NumProposalsForAccount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// Query Agreement which is approved and has escrow funds locked
	#[pallet::storage]
	#[pallet::getter(fn approved_service_agreements)]
	pub type ApprovedServiceAgreements<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AgreementId<T>,
		ServiceAgreement<T::AccountId>,
		OptionQuery,
	>;

	/// Query total number of agreements ever created used to get AgreementId Hash
	#[pallet::storage]
	#[pallet::getter(fn agreement_count)]
	pub type AgreementCount<T> = StorageValue<_, u128, ValueQuery>;

	//Index -> BUCKET_NAME/agreements
	#[pallet::storage]
	#[pallet::getter(fn remote_storage_index)]
	pub type RemoteStorageIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, Vec<u8>, OptionQuery>;

	//AgreementId -> Vec<(StorageBackendIndex, Suffix, ContentHash)>
	#[pallet::storage]
	#[pallet::getter(fn remote_storage_path)]
	pub type RemoteStoragePath<T: Config> =
		StorageMap<_, Blake2_128Concat, AgreementId<T>, RemoteStorage, OptionQuery>;

	// Predefined set of prefix URLs at genesis to be referenced via index
	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub prefix: Vec<u8>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<RemoteStorageIndex<T>>::insert(0, "GCP_BUCKET/agreements".encode());
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AgreementProposed {
			proposer: T::AccountId,
			buyer: T::AccountId,
			seller: T::AccountId,
			agreement_id: AgreementId<T>,
			agreement: ServiceAgreement<T::AccountId>,
		},
		AgreementApproved {
			agreement_id: AgreementId<T>,
			agreement: ServiceAgreement<T::AccountId>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Thrown when agreement proposer isnt a participant of the proposed agreement
		ProposerNotParticipant,
		/// Total cap on agreements is reached
		MaxAgreementsReached,
		/// Every user can have a capped amount of proposals they can be engaged in
		MaxProposalsExceededForUser,
		/// Thrown when someone is trying to sign/vote on a proposal which they are not in
		ApproverNotValidParticipant,
		/// Thrown when trying to approve a proposal which does not exist
		AgreementNotProposed,
		/// Thrown when a user is trying to double sign/vote for the same proposal
		DoubleVote,
		/// Thrown when trying to sign/vote for a proposal which has already passed
		ApprovalAlreadyAchievedForProposal,
		/// Thrown when the wrong storage index provided is unknown
		UnknownRemoteStorageIndex,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: frame_system::Config<Hash = sp_core::H256>,
	{
		/// Propose an agreement to be signed and agreed upon.
		///
		/// Checks:
		/// - If proposer is either the seller or buyer
		/// - Make sure atleast One Buyer and One Seller
		/// - Place in ServiceAgreementProposals
		///
		#[pallet::weight(0)]
		pub fn propose_agreement(
			origin: OriginFor<T>,
			agreement: ServiceAgreement<T::AccountId>,
		) -> DispatchResult {
			// Get Account of Signer
			let proposer = ensure_signed(origin)?;
			// Make sure proposer can propose and is not over max proposals
			ensure!(
				NumProposalsForAccount::<T>::get(&proposer) < T::MaxProposalsForUser::get(),
				Error::<T>::MaxProposalsExceededForUser,
			);
			// Get Buyer and Seller
			let buyer = agreement.try_get_buyer()?;
			let seller = agreement.try_get_seller()?;
			// Make sure signature matches a Participant
			let proposer_found = agreement
				.participants
				.iter()
				.find(|&participant| participant.account_id == proposer);
			ensure!(proposer_found.is_some(), Error::<T>::ProposerNotParticipant);
			// Create AgreementId by Hashing counter value for proposals
			let future_count =
				Self::agreement_count().checked_add(1).ok_or(Error::<T>::MaxAgreementsReached)?;
			AgreementCount::<T>::mutate(|count| *count = future_count);

			// Setting new agreement_id for agreement which will be stored
			// Which is the hash of the current service agreement count
			let agreement_id = T::Hash::from(T::Hashing::hash_of(&future_count));
			let mut agreement_to_be_stored = agreement.clone();
			agreement_to_be_stored.agreement_id = agreement_id.clone();
			// Place in ServiceAgreementProposals
			ProposedServiceAgreements::<T>::insert(&agreement_id, agreement_to_be_stored.clone());
			NumProposalsForAccount::<T>::mutate(&proposer, |count| *count = *count + 1);

			// Construct remote storage path
			let prefix = 0u128;
			RemoteStorageIndex::<T>::get(prefix).ok_or(Error::<T>::UnknownRemoteStorageIndex)?;
			let storage_path: RemoteStorage =
				RemoteStorage { prefix, suffix: future_count, agreement_id };
			RemoteStoragePath::<T>::insert(&agreement_id, storage_path);

			Self::deposit_event(Event::<T>::AgreementProposed {
				proposer,
				buyer: buyer.account_id,
				seller: seller.account_id,
				agreement_id,
				agreement: agreement_to_be_stored,
			});
			Ok(())
		}

		/// Signs the corresponding Agreement proposal.
		///
		/// Checks:
		/// - can only approve if origin is one of the agreement participants
		/// - a participant can only vote once
		/// - atleast one buyer and seller must vote
		/// - Check number of votes for agreement is below the threshold for approval
		/// - If num of votes for agreement is met then enact escrow
		///
		#[pallet::weight(0)]
		pub fn approve_agreement(
			origin: OriginFor<T>,
			agreement_id: AgreementId<T>,
		) -> DispatchResult {
			// Get Account of Signer
			let approver = ensure_signed(origin)?;
			// Check agreement was validly proposed
			let agreement = ProposedServiceAgreements::<T>::get(&agreement_id)
				.ok_or(Error::<T>::AgreementNotProposed)?;
			// Check approver is a participant in the agreement
			let approver_found = agreement
				.participants
				.iter()
				.find(|&participant| participant.account_id == approver);
			ensure!(approver_found.is_some(), Error::<T>::ApproverNotValidParticipant);
			// Check for Double voting
			let proposal_votes_for_user = ProposalVotesForAccount::<T>::get(&approver);
			let proposal_vote_status =
				proposal_votes_for_user.iter().find(|&&proposal| proposal == agreement_id);
			ensure!(proposal_vote_status.is_none(), Error::<T>::DoubleVote);

			if let Some(mut vote_info) = Self::proposal_votes(&agreement_id) {
				Self::check_vote_info(&approver, &agreement_id, &mut vote_info, &agreement)?;
			} else {
				// Brand new proposal vote
				ProposalVotes::<T>::mutate(&agreement_id, |info| {
					*info = Some(VoteInfo::<T::AccountId> {
						accounts_voted: vec![approver.clone()],
						total_votes: 1,
					})
				});
			}

			// Add to users proposals
			ProposalVotesForAccount::<T>::mutate(&approver, |proposals| {
				proposals.push(agreement_id.clone())
			});

			Ok(())
		}

		/// Pushes approved agreement through to escrow
		/// Checks:
		/// - Agreement is approved
		/// Action:
		/// - Remove Agreement from Proposed Agreements and Cleanup Storage
		///
		#[pallet::weight(0)]
		pub fn start_escrow_on_approved_agreement(
			_origin: OriginFor<T>,
			_agreement_id: AgreementId<T>,
		) -> DispatchResult {
			// Todo implement
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check the current vote info to take action and approve proposed agreement
		/// or update.
		/// Checks:
		/// - If agreement proposal already has votes necessary to pass
		/// - If agreement proposal has enough votes to be approved
		/// Actions:
		/// - Increment total_votes for this agreement
		/// - Approve the agreement move to approved agreements storage
		/// - Throw agreement passed event
		fn check_vote_info(
			approver: &T::AccountId,
			agreement_id: &AgreementId<T>,
			vote_info: &mut VoteInfo<T::AccountId>,
			agreement: &ServiceAgreement<T::AccountId>,
		) -> DispatchResult {
			let required_pass_votes = T::NumVotesForApproval::get();
			ensure!(
				vote_info.total_votes < required_pass_votes,
				Error::<T>::ApprovalAlreadyAchievedForProposal
			);
			// Should not need to worry about overflow because of previous check
			vote_info.total_votes += 1;
			vote_info.accounts_voted.push(approver.clone());
			ProposalVotes::<T>::mutate(&agreement_id, |info| *info = Some(vote_info.clone()));
			if vote_info.total_votes >= required_pass_votes {
				ApprovedServiceAgreements::<T>::insert(&agreement_id, agreement.clone());
				Self::deposit_event(Event::<T>::AgreementApproved {
					agreement_id: *agreement_id,
					agreement: agreement.clone(),
				});
			}

			Ok(())
		}
	}
}
