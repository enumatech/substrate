/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;
use {erc20_multi};
use parity_codec::{Encode, Decode};
// use crate::traits::{Member, SimpleArithmetic, MaybeDebug};

#[derive(Copy, Debug, Encode, Decode, Clone, Eq, PartialEq)]
pub enum Intent {
	BuyAll,
	SellAll,
}

// type Round = u32; // unused
pub type Timestamp = u64;
pub type TokenId = u32; // from erc20-multi srml
// Ethereum compat:
// pub type Address = ::primitives::H160;
pub type ApprovalId = ::primitives::H256;

#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct Lot<Amount> {//where Amount: T::TokenBalance {
	pub asset: TokenId,
	pub amount: Amount,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct Approval<Amount, AccountId> {//where AccountId = <T as system::Trait>::AccountId {
	// round: Round; // Round when the approval is created/valid
	pub approval_id: ApprovalId, // Unique identifier of the approval
	pub buy: Lot<Amount>, // Buy side of the approval
	pub sell: Lot<Amount>, // Sell side of the approval
	pub intent: Intent, // Intent of the approval
	pub owner: AccountId, // Address of the creator of the approval
	pub timestamp: Timestamp, // Time when the approval is created
	// pub instance_id: Address, // Address of the mediator (mock)
}

// struct SignedApproval<T: Trait> {
// 	params: Approval<T>,
// 	ownerSig: T::Signature,
// }

/// The module's configuration trait.
pub trait Trait: erc20_multi::Trait {
	// TODO: Add other types and constants required configure this module.
	// type Approval: Approval<Self>;
	// type SignedApproval: SignedApproval<Self>;
	// type Amount: Member + Parameter + SimpleArithmetic + Default + Copy + MaybeDebug;

	// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(something): Option<u32>;

		// Filled: map ApprovalHash => (Amount, Amount);
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		// Just a dummy entry point.
		// function that can be called by the external world as an extrinsics call
		// takes a parameter of the type `AccountId`, stores it and emits an event
		pub fn do_something(origin, something: u32) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;

			// TODO: Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			<Something<T>>::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		pub fn fill(origin, a: Approval<T::TokenBalance, T::AccountId>, b: Approval<T::TokenBalance, T::AccountId>) -> Result {
			let who = ensure_signed(origin)?;
	    	Self::_fill(who, a, b)
		}

		pub fn batchFill(origin, batch: Vec<(Approval<T::TokenBalance, T::AccountId>, Approval<T::TokenBalance, T::AccountId>)>) -> Result {
			let who = ensure_signed(origin)?;
			// FIXME: fail the batch (rollback?) on any error
			// batch.into_iter().for_each(|(a,b)| Self::_fill(who, a, b));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),

		Filled(/*todo*/),
	}
);

// implementation of mudule
// utility and private functions
// if marked public, accessible by other modules
impl<T: Trait> Module<T> {
    // internal
    fn _fill(who: T::AccountId, a: Approval<T::TokenBalance, T::AccountId>, b: Approval<T::TokenBalance, T::AccountId>) -> Result {
		// TODO: Check A sig
		// TODO: Check B sig

		// Check unique IDs
		if a.approval_id == b.approval_id {
			return Err("need unique approvals")
		}

		// Check unique owners
		if a.owner == b.owner {
			return Err("need unique owner")
		}

		// Check compatible assets
		if a.buy.asset != b.sell.asset || a.sell.asset != b.buy.asset {
			return Err("incompatible assets");
		}

		// Find previous settlements, if any
		// let (aB_, aS_) = <Settled<T>>::get(A.approvalId);
		// let (bB_, bS_) = <Settled<T>>::get(B.approvalId);
		// ensure!(aB_ < A.buy.amount, "Nothing left to buy for A");
		// ensure!(bB_ < B.buy.amount, "Nothing left to buy for B");
		// ensure!(aS_ < A.sell.amount, "Nothing left to sell for A");
		// ensure!(bS_ < B.sell.amount, "Nothing left to sell for B");
		// let aB = A.buy.amount - aB_;
		// let aS = A.sell.amount - aS_;
		// let bB = B.buy.amount - bB_;
		// let bS = B.sell.amount - bS_;

		// TEMP: ensure exact match
		if a.buy.amount != b.sell.amount || a.sell.amount != b.buy.amount {
			// TODO: calc fillable amounts
			return Err("incompatible amounts");
		}

		// TODO: this doesn't actually involve any cryptography!
		let new_origin = system::RawOrigin::Signed(who.clone()).into();
		if let Err(e) = <erc20_multi::Module<T>>::transfer_from(new_origin, a.sell.asset, a.owner.clone(), b.owner.clone(), a.sell.amount) {
			return Err(e);
		}
		let new_origin2 = system::RawOrigin::Signed(who).into();
		if let Err(e) = <erc20_multi::Module<T>>::transfer_from(new_origin2, b.sell.asset, b.owner, a.owner, b.sell.amount) {
			// FIXME: rollback?
			return Err(e);
		}

		// here we are raising the Filled event
		Self::deposit_event(RawEvent::Filled(/*a.approval_id, b.approval_id*/));
		Ok(())
    }
}


/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, assert_noop};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};
	// use std::fmt;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// impl fmt::Debug for assets::Trait::Balance {
	// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	// 		write!(f, "Balance {{ .. }}")
	// 	}
	// }

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl erc20_multi::Trait for Test {
		type Event = ();
    	type TokenBalance = u128;
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
		// let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
		// t.extend(erc20_multi::GenesisConfig::<Test>{
		// 	code: vec![],
		// 	authorities: authorities.into_iter().map(|a| UintAuthorityId(a)).collect(),
		// }.build_storage().unwrap().0);
		// t.extend(timestamp::GenesisConfig::<Test>{
		// 	minimum_period: 1,
		// }.build_storage().unwrap().0);
		// t.into()
	}

	const A: TokenId = 0;
	const B: TokenId = 1;
	const ALICE: u64 = 1;
	const BOB: u64 = 2;
	const DEX: u64 = 3;
	type Erc20 = erc20_multi::Module<Test>;

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}

	fn make_approval<TB, AC>(id: u8, buy: Lot<TB>, sell: Lot<TB>, owner: AC, intent: Intent) -> Approval<TB, AC> {
		let seed: [u8; 32] = [
			id, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0,
		];
		Approval {
			approval_id: H256::from_slice(&seed),
			buy: buy,
			sell: sell,
			intent: intent,
			owner: owner,
			timestamp: 0,
			// instance_id: ::primitives::H160::default(),
		}
	}

	#[test]
	fn it_cannot_fill_without_allowance() {
		with_externalities(&mut new_test_ext(), || {
			let a = make_approval(0, Lot { amount: 1, asset: B}, Lot { amount: 1, asset: A}, ALICE, Intent::BuyAll);
			let b = make_approval(1, Lot { amount: 1, asset: A}, Lot { amount: 1, asset: B}, BOB, Intent::BuyAll);
			assert_noop!(TemplateModule::fill(Origin::signed(DEX), a, b), "Allowance does not exist.");
		});
	}

	#[test]
	fn it_can_fill_with_allowance() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(Erc20::init(Origin::signed(ALICE), b"AliceCoin".to_vec(), b"A".to_vec(), 1));
			assert_ok!(Erc20::init(Origin::signed(BOB), b"BobCoin".to_vec(), b"B".to_vec(), 1));
			assert_ok!(Erc20::approve(Origin::signed(ALICE), A, DEX, 1));
			assert_ok!(Erc20::approve(Origin::signed(BOB), B, DEX, 1));
			let a = make_approval(0, Lot { amount: 1, asset: B}, Lot { amount: 1, asset: A}, ALICE, Intent::BuyAll);
			let b = make_approval(1, Lot { amount: 1, asset: A}, Lot { amount: 1, asset: B}, BOB, Intent::BuyAll);
			assert_ok!(TemplateModule::fill(Origin::signed(DEX), a, b));
			assert_eq!(Erc20::balance_of((A, ALICE)), 0);
			assert_eq!(Erc20::balance_of((B, ALICE)), 1);
			assert_eq!(Erc20::balance_of((A, BOB)), 1);
			assert_eq!(Erc20::balance_of((B, BOB)), 0);
		});
	}

	#[test]
	fn it_is_atomic_1() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(Erc20::init(Origin::signed(ALICE), b"AliceCoin".to_vec(), b"A".to_vec(), 1));
			assert_ok!(Erc20::init(Origin::signed(BOB), b"BobCoin".to_vec(), b"B".to_vec(), 1));
			assert_ok!(Erc20::approve(Origin::signed(ALICE), A, DEX, 1));
			// assert_ok!(Erc20::approve(Origin::signed(BOB), B, DEX, 1)); skip
			let a = make_approval(0, Lot { amount: 1, asset: B}, Lot { amount: 1, asset: A}, ALICE, Intent::BuyAll);
			let b = make_approval(1, Lot { amount: 1, asset: A}, Lot { amount: 1, asset: B}, BOB, Intent::BuyAll);
			assert_noop!(TemplateModule::fill(Origin::signed(DEX), a, b), "Allowance does not exist.");
			assert_eq!(Erc20::balance_of((A, ALICE)), 1);//fails!!
			assert_eq!(Erc20::balance_of((B, ALICE)), 0);
			assert_eq!(Erc20::balance_of((A, BOB)), 0);
			assert_eq!(Erc20::balance_of((B, BOB)), 1);
		});
	}

	#[test]
	fn it_is_atomic_2() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(Erc20::init(Origin::signed(ALICE), b"AliceCoin".to_vec(), b"A".to_vec(), 1));
			assert_ok!(Erc20::init(Origin::signed(BOB), b"BobCoin".to_vec(), b"B".to_vec(), 1));
			// assert_ok!(Erc20::approve(Origin::signed(ALICE), A, DEX, 1)); skip
			assert_ok!(Erc20::approve(Origin::signed(BOB), B, DEX, 1));
			let a = make_approval(0, Lot { amount: 1, asset: B}, Lot { amount: 1, asset: A}, ALICE, Intent::BuyAll);
			let b = make_approval(1, Lot { amount: 1, asset: A}, Lot { amount: 1, asset: B}, BOB, Intent::BuyAll);
			assert_noop!(TemplateModule::fill(Origin::signed(DEX), a, b), "Allowance does not exist.");
			assert_eq!(Erc20::balance_of((A, ALICE)), 1);
			assert_eq!(Erc20::balance_of((B, ALICE)), 0);
			assert_eq!(Erc20::balance_of((A, BOB)), 0);
			assert_eq!(Erc20::balance_of((B, BOB)), 1);
		});
	}
}
