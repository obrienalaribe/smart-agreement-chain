use scale_info::TypeInfo;
use codec::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use frame_support::sp_runtime::{RuntimeDebug};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct AgreementInfo<AccountId, AgreementId> {
	pub party_a: AccountId,
    pub party_b: AccountId,
    pub agreement_id: AgreementId,
    pub agreement_type: AgreementType,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub enum AgreementType {
    #[codec(index = 1)]
    ServiceAgreement,
    //..
}

impl Default for AgreementType {
    fn default() -> Self {
        AgreementType::ServiceAgreement
    }
}