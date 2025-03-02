#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

mod types;
use types::*;

use frame_system::pallet_prelude::*;
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency, LockableCurrency,
		ExistenceRequirement::KeepAlive,
	},
	transactional,
};
use cp_cess_common::IpAddress;

pub use pallet::*;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use crate::*;
	use frame_system::ensure_signed;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency trait.
		type Currency: LockableCurrency<Self::AccountId>;

		/// The maximum length of bill list when calling the pay function.
		#[pallet::constant]
		type BillsLimit: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//The event of successful Cacher registration
		Register { acc: AccountOf<T>, info: CacherInfo<AccountOf<T>, BalanceOf<T>> },
		//Cacher information change success event
		Update { acc: AccountOf<T>, info: CacherInfo<AccountOf<T>, BalanceOf<T>> },
		//Cacher account logout success event
		Logout { acc: AccountOf<T> },
		//Pay to cacher success event
		Pay { acc: AccountOf<T>, bills: BoundedVec<Bill<AccountOf<T>, BalanceOf<T>, T::Hash>, T::BillsLimit> },
	}

	#[pallet::error]
	pub enum Error<T> {
		//Registered Error
		AlreadyRegistered,
		//Unregistered Error
		UnRegistered,
		//Option parse Error
		OptionParseError,
	}

	/// Store all cacher info
	#[pallet::storage]
	#[pallet::getter(fn cacher)]
	pub(super) type Cachers<T: Config> = StorageMap<_, Blake2_128Concat, AccountOf<T>, CacherInfo<AccountOf<T>, BalanceOf<T>>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Register for cacher.
		///	
		/// Parameters:
		/// - `info`: The cacher info related to signer account.
		#[pallet::weight(T::WeightInfo::register())]
		pub fn register(origin: OriginFor<T>, info: CacherInfo<AccountOf<T>, BalanceOf<T>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!<Cachers<T>>::contains_key(&sender), Error::<T>::AlreadyRegistered);
			<Cachers<T>>::insert(&sender, info.clone());

			Self::deposit_event(Event::<T>::Register {acc: sender, info});

			Ok(())
		}

		/// Update cacher info.
		///	
		/// Parameters:
		/// - `info`: The cacher info related to signer account.
		#[pallet::weight(T::WeightInfo::update())]
		pub fn update(origin: OriginFor<T>, info: CacherInfo<AccountOf<T>, BalanceOf<T>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Cachers<T>>::contains_key(&sender), Error::<T>::UnRegistered);

			<Cachers<T>>::try_mutate(&sender, |info_opt| -> DispatchResult {
				let p_info = info_opt.as_mut().ok_or(Error::<T>::OptionParseError)?;
				*p_info = info.clone();
				Ok(())
			})?;

			Self::deposit_event(Event::<T>::Update {acc: sender, info});

			Ok(())
		}

		/// Cacher exit method, Irreversible process.
		#[pallet::weight(T::WeightInfo::logout())]
		pub fn logout(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Cachers<T>>::contains_key(&sender), Error::<T>::UnRegistered);

			<Cachers<T>>::remove(&sender);

			Self::deposit_event(Event::<T>::Logout { acc: sender });

			Ok(())
		}

		/// Pay to cachers for downloading files.
		///	
		/// Parameters:
		/// - `bills`: list of bill.
		#[transactional]
		#[pallet::weight(T::WeightInfo::pay(bills.len() as u32))]
		pub fn pay(origin: OriginFor<T>, bills: BoundedVec<Bill<AccountOf<T>, BalanceOf<T>, T::Hash>, T::BillsLimit>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			for bill in bills.clone() {
				T::Currency::transfer(&sender, &bill.to, bill.amount, KeepAlive)?;
			}
			
			Self::deposit_event(Event::<T>::Pay { acc: sender, bills });

			Ok(())
		}
	}
}
