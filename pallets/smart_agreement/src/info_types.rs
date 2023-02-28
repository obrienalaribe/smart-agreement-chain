use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_runtime::RuntimeDebug;

use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_std::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Participant<AccountId> {
	pub account_id: AccountId,
	/// A party who is providing a service or product for an agreement.
	pub role: Role,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Role {
	/// A party who is receiving a service or product from an agreement.
	Buyer,
	/// A party who is providing a service or product for an agreement.
	Seller,
}

impl<AccountId> Participant<AccountId> {
	pub fn new(role: Role, account_id: AccountId) -> Self {
		Self { role, account_id }
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct VoteInfo<AccountId> {
	pub accounts_voted: Vec<AccountId>,
	pub total_votes: u32,
}

pub trait AgreementInfo<AccountId> {
	/// parties who are partaking in an agreement
	/// - Must have atleast one buyer and seller
	type Participants;
	/// Party which is buying service or product
	type Buyer;
	/// Party providing service or product
	type Seller;
	/// Identfication of a particular agreement for query lookup
	type AgreementId;
	/// An amount of currency to Lock into an Escrow until a Service has been
	/// completed.
	type Amount; // Option<Currency>
	/// A hash of the Agreement Data.
	type AgreementDataHash;

	/// If possible get the buyer from the participants
	fn try_get_buyer(&self) -> Result<Participant<AccountId>, sp_runtime::DispatchError>;
	/// If possible get the seller from the participants
	fn try_get_seller(&self) -> Result<Participant<AccountId>, sp_runtime::DispatchError>;
	/// Get the Amount of currency in this agreement
	fn amount(&self) -> Self::Amount;
	/// The hash of the scale_encoded bytes of the agreement.
	fn agreement_data_hash(&self) -> Self::AgreementDataHash;
	/// The Identifier of this Agreement amongst all the agreements
	fn agreement_id(&self) -> Self::AgreementId;
	/// participants who are engaged in this agreement
	fn participants(&self) -> Self::Participants;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, TypeInfo, Debug)]
pub struct ServiceAgreement<AccountId> {
	pub participants: Vec<Participant<AccountId>>,
	pub agreement_id: H256,
	pub amount: Option<u64>,
	pub agreement_data_hash: H256,
}

impl<AccountId: Clone> AgreementInfo<AccountId> for ServiceAgreement<AccountId> {
	type Participants = Vec<Participant<AccountId>>;
	type AgreementId = H256;
	type Amount = Option<u64>;
	type AgreementDataHash = H256;
	type Buyer = Result<Participant<AccountId>, sp_runtime::DispatchError>;
	type Seller = Result<Participant<AccountId>, sp_runtime::DispatchError>;

	fn try_get_buyer(&self) -> Self::Buyer {
		for participant in self.participants.iter() {
			match participant.role {
				Role::Buyer => return Ok(participant.clone()),
				Role::Seller => {},
			}
		}
		Err(sp_runtime::DispatchError::Other("No Buyer in Agreement"))
	}
	fn try_get_seller(&self) -> Self::Seller {
		for participant in self.participants.iter() {
			match participant.role {
				Role::Buyer => {},
				Role::Seller => return Ok(participant.clone()),
			}
		}
		Err(sp_runtime::DispatchError::Other("No Seller in Agreement"))
	}
	fn amount(&self) -> Self::Amount {
		self.amount
	}
	fn agreement_data_hash(&self) -> Self::AgreementDataHash {
		self.agreement_data_hash
	}
	fn agreement_id(&self) -> Self::AgreementId {
		self.agreement_id
	}
	fn participants(&self) -> Self::Participants {
		self.participants.clone()
	}
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct RemoteStorage {
	pub prefix: u128,
	pub suffix: u128,
	pub agreement_id: H256,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
#[repr(u32)]
pub enum RemoteStorageProvider {
	GCS = 0,
}
