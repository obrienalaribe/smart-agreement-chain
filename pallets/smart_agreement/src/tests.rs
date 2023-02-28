use crate::{info_types::*, mock::*, Error, RemoteStorageIndex, RemoteStoragePath};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::{testing::SR25519, H256};
use sp_runtime::traits::Hash;

pub type HashType = <Test as frame_system::Config>::Hash;
pub type Hashing = <Test as frame_system::Config>::Hashing;
pub type AccountId = <Test as frame_system::Config>::AccountId;

fn service_agreement_default<AccountId>() -> ServiceAgreement<AccountId>
where
	AccountId: From<sp_core::sr25519::Public>,
{
	let alice_pub: AccountId = sp_io::crypto::sr25519_public_keys(SR25519)[0].into();
	let bob_pub: AccountId = sp_io::crypto::sr25519_public_keys(SR25519)[1].into();
	let alice = Participant::<AccountId>::new(Role::Buyer, alice_pub);
	let bob = Participant::<AccountId>::new(Role::Seller, bob_pub);
	ServiceAgreement::<AccountId> {
		participants: vec![alice, bob],
		agreement_id: H256::default(),
		amount: None,
		agreement_data_hash: H256::default(),
	}
}

#[test]
fn base_prefix_for_remote_storage() {
	new_test_ext().execute_with(|| {
		assert_eq!(RemoteStorageIndex::<Test>::get(0).unwrap(), "GCP_BUCKET/agreements".encode());
		assert_eq!(RemoteStorageIndex::<Test>::get(1), None);
	})
}

#[test]
fn create_default_agreement() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let bob_pub = sp_io::crypto::sr25519_public_keys(SR25519)[1];
		let alice = Participant::<AccountId>::new(Role::Buyer, alice_pub);
		let bob = Participant::<AccountId>::new(Role::Seller, bob_pub);

		let sa = service_agreement_default::<AccountId>();
		let buyer = sa.try_get_buyer().unwrap();
		let seller = sa.try_get_seller().unwrap();
		assert_eq!(buyer, alice);
		assert_eq!(seller, bob);
	})
}

#[test]
fn propose_agreement() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let mut sa = service_agreement_default::<AccountId>();
		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa.clone()));

		const AGREEMENT_COUNT: u128 = 1;
		assert_eq!(SmartAgreement::agreement_count(), AGREEMENT_COUNT);

		let agreement_id = HashType::from(Hashing::hash_of(&AGREEMENT_COUNT));
		sa.agreement_id = agreement_id.clone();
		assert_eq!(SmartAgreement::proposed_service_agreements(&agreement_id).unwrap(), sa);
		assert_eq!(SmartAgreement::num_proposals_for_account(alice_pub), AGREEMENT_COUNT as u32);
		assert_eq!(
			RemoteStoragePath::<Test>::get(&agreement_id),
			Some(RemoteStorage { prefix: 0, suffix: AGREEMENT_COUNT, agreement_id })
		);
	})
}

#[test]
fn max_proposals_reached_for_user_fails() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let sa = service_agreement_default::<AccountId>();
		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa.clone()));
		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa.clone()));
		assert_noop!(
			SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa.clone()),
			Error::<Test>::MaxProposalsExceededForUser
		);
	})
}

#[test]
fn proposal_when_not_participant_fails() {
	new_test_ext().execute_with(|| {
		let eve_pub = sp_io::crypto::sr25519_public_keys(SR25519)[2];
		let sa = service_agreement_default::<AccountId>();
		assert_noop!(
			SmartAgreement::propose_agreement(Origin::signed(eve_pub), sa),
			Error::<Test>::ProposerNotParticipant
		);
	})
}

#[test]
fn proposal_with_no_buyer_fails() {
	new_test_ext().execute_with(|| {
		let eve_pub = sp_io::crypto::sr25519_public_keys(SR25519)[2];
		let mut sa = service_agreement_default::<AccountId>();
		// The Buyer is the first element in the test_default agreement participant list
		sa.participants.remove(0);
		assert_noop!(
			SmartAgreement::propose_agreement(Origin::signed(eve_pub), sa),
			sp_runtime::DispatchError::Other("No Buyer in Agreement")
		);
	})
}

#[test]
fn proposal_with_no_seller_fails() {
	new_test_ext().execute_with(|| {
		let eve_pub = sp_io::crypto::sr25519_public_keys(SR25519)[2];
		let mut sa = service_agreement_default::<AccountId>();
		// The Seller is the last element in the test_default agreement participant list
		sa.participants.pop();
		assert_noop!(
			SmartAgreement::propose_agreement(Origin::signed(eve_pub), sa),
			sp_runtime::DispatchError::Other("No Seller in Agreement")
		);
	})
}

#[test]
fn agreement_not_proposed_fails() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let agreement_id = HashType::from(Hashing::hash_of(&0));
		assert_noop!(
			SmartAgreement::approve_agreement(Origin::signed(alice_pub), agreement_id),
			Error::<Test>::AgreementNotProposed
		);
	})
}

#[test]
fn duplicate_approval_vote_fails() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let sa = service_agreement_default::<AccountId>();
		const AGREEMENT_COUNT: u128 = 1;
		let agreement_id = HashType::from(Hashing::hash_of(&AGREEMENT_COUNT));
		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa));
		assert_ok!(SmartAgreement::approve_agreement(
			Origin::signed(alice_pub),
			agreement_id.clone()
		));
		assert_noop!(
			SmartAgreement::approve_agreement(Origin::signed(alice_pub), agreement_id),
			Error::<Test>::DoubleVote
		);
	})
}

#[test]
fn approver_not_participant_fails() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let eve_pub = sp_io::crypto::sr25519_public_keys(SR25519)[2];
		let sa = service_agreement_default::<AccountId>();
		const AGREEMENT_COUNT: u128 = 1;
		let agreement_id = HashType::from(Hashing::hash_of(&AGREEMENT_COUNT));
		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa));
		assert_noop!(
			SmartAgreement::approve_agreement(Origin::signed(eve_pub), agreement_id),
			Error::<Test>::ApproverNotValidParticipant
		);
	})
}

#[test]
fn approving_proposal_when_already_approved_fails() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let bob_pub = sp_io::crypto::sr25519_public_keys(SR25519)[1];
		let eve_pub = sp_io::crypto::sr25519_public_keys(SR25519)[2];
		let eve = Participant::<AccountId>::new(Role::Buyer, eve_pub);
		let mut sa = service_agreement_default::<AccountId>();
		// Default only has two Participants need atleast 3 for this test
		sa.participants.push(eve);
		const AGREEMENT_COUNT: u128 = 1;
		let agreement_id = HashType::from(Hashing::hash_of(&AGREEMENT_COUNT));

		assert_ok!(SmartAgreement::propose_agreement(Origin::signed(alice_pub), sa));
		assert_ok!(SmartAgreement::approve_agreement(
			Origin::signed(alice_pub),
			agreement_id.clone()
		));
		assert_ok!(SmartAgreement::approve_agreement(
			Origin::signed(bob_pub),
			agreement_id.clone()
		));

		assert_noop!(
			SmartAgreement::approve_agreement(Origin::signed(eve_pub), agreement_id.clone()),
			Error::<Test>::ApprovalAlreadyAchievedForProposal
		);
	})
}

#[test]
fn approval_reached() {
	new_test_ext().execute_with(|| {
		let alice_pub = sp_io::crypto::sr25519_public_keys(SR25519)[0];
		let bob_pub = sp_io::crypto::sr25519_public_keys(SR25519)[1];
		let mut sa = service_agreement_default::<AccountId>();

		const AGREEMENT_COUNT: u128 = 1;
		let agreement_id = HashType::from(Hashing::hash_of(&AGREEMENT_COUNT));

		assert_ok!(SmartAgreement::propose_agreement(
			Origin::signed(alice_pub.clone()),
			sa.clone()
		));
		sa.agreement_id = agreement_id.clone();

		assert_ok!(SmartAgreement::approve_agreement(
			Origin::signed(alice_pub.clone()),
			agreement_id.clone()
		));
		assert_eq!(
			SmartAgreement::proposal_votes_for_account(&alice_pub),
			vec![agreement_id.clone()]
		);

		let mut vote_info =
			VoteInfo::<AccountId> { accounts_voted: vec![alice_pub], total_votes: 1 };
		assert_eq!(SmartAgreement::proposal_votes(&agreement_id).unwrap(), vote_info.clone());

		assert_ok!(SmartAgreement::approve_agreement(
			Origin::signed(bob_pub.clone()),
			agreement_id.clone()
		));
		assert_eq!(
			SmartAgreement::proposal_votes_for_account(&bob_pub),
			vec![agreement_id.clone()]
		);

		vote_info.accounts_voted.push(bob_pub.clone());
		vote_info.total_votes += 1;
		assert_eq!(
			SmartAgreement::proposal_votes(&agreement_id)
				.expect("If nothing here then is bad we want to fail; QED"),
			vote_info
		);

		assert_eq!(
			SmartAgreement::approved_service_agreements(&agreement_id)
				.expect("If nothing here then is bad we want to fail; QED"),
			sa
		);
	})
}
