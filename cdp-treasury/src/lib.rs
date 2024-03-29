// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of Ethical DeFi.
//
// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)

//! # CDP Treasury Module
//!
//! ## Overview
//!
//! CDP Treasury manages bad debts generated by
//! CDPs, and handle excessive surplus or debits timely in order to keep the
//! system healthy with low risk. 

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{log, pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	ArithmeticError, DispatchError, DispatchResult, FixedPointNumber,
};
use support::{AuctionManager, CDPTreasury, CDPTreasuryExtended, DEXManager, Ratio, SerpTreasury, SwapLimit};
use sp_std::{prelude::*, vec};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency for managing assets related to CDP
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Stablecoin currency id
		#[pallet::constant]
		type GetSetUSDId: Get<CurrencyId>;

		/// SERP Treasury for issuing/burning stable currency adjust standard value
		/// adjustment
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// Auction manager creates auction to handle system surplus and debit
		type AuctionManagerHandler: AuctionManager<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Dex manager is used to swap confiscated collateral assets to stable
		/// currency
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The cap of lots number when create collateral auction on a
		/// liquidation or to create debit/surplus auction on block end.
		/// If set to 0, does not work.
		#[pallet::constant]
		type MaxAuctionsCount: Get<u32>;

		/// The origin which may update parameters and handle
		/// surplus/collateral.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// The CDP treasury's module id, keep surplus and collateral assets
		/// from liquidation.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The alternative swap path joint list, which can be concated to
		/// alternative swap path when cdp treasury swap collateral to stable.
		#[pallet::constant]
		type AlternativeSwapPathJointList: Get<Vec<Vec<CurrencyId>>>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The collateral amount of CDP treasury is not enough
		CollateralNotEnough,
		/// The surplus pool of CDP treasury is not enough
		SurplusPoolNotEnough,
		/// The debit pool of CDP treasury is not enough
		DebitPoolNotEnough,
		/// Cannot use collateral to swap stable
		CannotSwap,
		/// The currency id is not DexShare type
		NotDexShare,
	}


	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The expected amount size for per lot collateral auction of specific collateral type
		/// updated.
		ExpectedCollateralAuctionSizeUpdated {
			collateral_type: CurrencyId,
			new_size: Balance,
		},
	}

	/// The expected amount size for per lot collateral auction of specific
	/// collateral type.
	///
	/// ExpectedCollateralAuctionSize: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn expected_collateral_auction_size)]
	pub type ExpectedCollateralAuctionSize<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// Current total debit value of system under `currency_id`. It's not same as debit in CDP
	/// engine, it is the bad debt of the system under a specific stable currency.
	///
	/// DebitPool: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn debit_pool)]
	pub type DebitPool<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub expected_collateral_auction_size: Vec<(CurrencyId, Balance)>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				expected_collateral_auction_size: vec![],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.expected_collateral_auction_size
				.iter()
				.for_each(|(currency_id, size)| {
					ExpectedCollateralAuctionSize::<T>::insert(currency_id, size);
				});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Handle excessive surplus or debits of system when block end
		fn on_finalize(_now: T::BlockNumber) {
			// offset the same amount between debit pool and surplus pool
			Self::offset_surplus_and_debit();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Extract surplus to SerpTreasury.
		#[pallet::weight(T::WeightInfo::extract_surplus_to_serp())]
		#[transactional]
		pub fn extract_surplus_to_serp(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			T::SerpTreasury::on_serplus(T::GetSetUSDId::get(), amount)?;
			Ok(().into())
		}

		/// Auction the collateral not occupied by the auction.
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: collateral type
		/// - `amount`: collateral amount
		/// - `target`: target amount
		/// - `splited`: splite collateral to multiple auction according to the config size
		#[pallet::weight(
			if *splited {
				T::WeightInfo::auction_collateral(T::MaxAuctionsCount::get())
			} else {
				T::WeightInfo::auction_collateral(1)
			}
		)]
		#[transactional]
		pub fn auction_collateral(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			#[pallet::compact] amount: Balance,
			#[pallet::compact] target: Balance,
			splited: bool,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			<Self as CDPTreasuryExtended<T::AccountId>>::create_collateral_auctions(
				currency_id,
				amount,
				target,
				Self::account_id(),
				splited,
			)?;
			Ok(())
		}

		/// Swap the collateral not occupied by the auction to stable.
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: collateral type
		/// - `swap_limit`: target amount
		#[pallet::weight(T::WeightInfo::exchange_collateral_to_stable())]
		#[transactional]
		pub fn exchange_collateral_to_stable(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			swap_limit: SwapLimit<Balance>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			// the supply collateral must not be occupied by the auction.
			Self::swap_collateral_to_stable(currency_id, swap_limit, false)?;
			Ok(())
		}

		/// Update parameters related to collateral auction under specific
		/// collateral type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: collateral type
		/// - `amount`: expected size of per lot collateral auction
		#[pallet::weight((T::WeightInfo::set_expected_collateral_auction_size(), DispatchClass::Operational))]
		#[transactional]
		pub fn set_expected_collateral_auction_size(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			#[pallet::compact] size: Balance,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			ExpectedCollateralAuctionSize::<T>::insert(currency_id, size);
			Self::deposit_event(Event::ExpectedCollateralAuctionSizeUpdated {
				collateral_type: currency_id,
				new_size: size,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of cdp treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// Get current total surplus of system.
	pub fn surplus_pool() -> Balance {
		T::Currency::free_balance(T::GetSetUSDId::get(), &Self::account_id())
	}

	/// Get total collateral amount of cdp treasury module.
	pub fn total_collaterals(currency_id: CurrencyId) -> Balance {
		T::Currency::free_balance(currency_id, &Self::account_id())
	}

	/// Get collateral amount not in auction
	pub fn total_collaterals_not_in_auction(currency_id: CurrencyId) -> Balance {
		T::Currency::free_balance(currency_id, &Self::account_id())
			.saturating_sub(T::AuctionManagerHandler::get_total_collateral_in_auction(currency_id))
	}

	fn offset_surplus_and_debit() {
		let offset_amount = sp_std::cmp::min(Self::debit_pool(), Self::surplus_pool());

		// Burn the amount that is equal to offset amount of stable currency.
		if !offset_amount.is_zero() {
			let res = T::Currency::withdraw(T::GetSetUSDId::get(), &Self::account_id(), offset_amount);
			match res {
				Ok(_) => {
					DebitPool::<T>::mutate(|debit| {
						*debit = debit
							.checked_sub(offset_amount)
							.expect("offset = min(debit, surplus); qed")
					});
				}
				Err(e) => {
					log::warn!(
						target: "cdp-treasury",
						"get_swap_supply_amount: Attempt to burn surplus {:?} failed: {:?}, this is unexpected but should be safe",
						offset_amount, e
					);
				}
			}
		}
	}
}

impl<T: Config> CDPTreasury<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	fn get_surplus_pool() -> Self::Balance {
		Self::surplus_pool()
	}

	fn get_debit_pool() -> Self::Balance {
		Self::debit_pool()
	}

	fn get_total_collaterals(id: Self::CurrencyId) -> Self::Balance {
		Self::total_collaterals(id)
	}

	fn get_debit_proportion(amount: Self::Balance) -> Ratio {
		let stable_total_supply = T::Currency::total_issuance(T::GetSetUSDId::get());
		Ratio::checked_from_rational(amount, stable_total_supply).unwrap_or_default()
	}

	fn on_system_debit(amount: Self::Balance) -> DispatchResult {
		DebitPool::<T>::try_mutate(|debit_pool| -> DispatchResult {
			*debit_pool = debit_pool.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
			Ok(())
		})
	}

	fn on_system_surplus(amount: Self::Balance) -> DispatchResult {
		Self::issue_debit(&Self::account_id(), amount, true)
	}

	fn issue_debit(who: &T::AccountId, debit: Self::Balance, backed: bool) -> DispatchResult {
		// increase system debit if the debit is unbacked
		if !backed {
			Self::on_system_debit(debit)?;
		}
		T::Currency::deposit(T::GetSetUSDId::get(), who, debit)?;

		Ok(())
	}

	fn burn_debit(who: &T::AccountId, debit: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::GetSetUSDId::get(), who, debit)
	}

	fn deposit_surplus(from: &T::AccountId, surplus: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::GetSetUSDId::get(), from, &Self::account_id(), surplus)
	}

	fn deposit_collateral(from: &T::AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, from, &Self::account_id(), amount)
	}

	fn withdraw_collateral(to: &T::AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, &Self::account_id(), to, amount)
	}
}

impl<T: Config> CDPTreasuryExtended<T::AccountId> for Pallet<T> {
	fn swap_collateral_to_stable(
		currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
		collateral_in_auction: bool,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		let supply_limit = match limit {
			SwapLimit::ExactSupply(supply_amount, _) => supply_amount,
			SwapLimit::ExactTarget(max_supply_amount, _) => max_supply_amount,
		};
		if collateral_in_auction {
			ensure!(
				Self::total_collaterals(currency_id) >= supply_limit
					&& T::AuctionManagerHandler::get_total_collateral_in_auction(currency_id) >= supply_limit,
				Error::<T>::CollateralNotEnough,
			);
		} else {
			ensure!(
				Self::total_collaterals_not_in_auction(currency_id) >= supply_limit,
				Error::<T>::CollateralNotEnough,
			);
		}

		let swap_path = T::DEX::get_best_price_swap_path(
			currency_id,
			T::GetSetUSDId::get(),
			limit,
			T::AlternativeSwapPathJointList::get(),
		)
		.ok_or(Error::<T>::CannotSwap)?;
		T::DEX::swap_with_specific_path(&Self::account_id(), &swap_path, limit)
	}

	fn create_collateral_auctions(
		currency_id: CurrencyId,
		amount: Balance,
		target: Balance,
		refund_receiver: T::AccountId,
		splited: bool,
	) -> DispatchResult {
		ensure!(
			Self::total_collaterals_not_in_auction(currency_id) >= amount,
			Error::<T>::CollateralNotEnough,
		);

		let mut unhandled_collateral_amount = amount;
		let mut unhandled_target = target;
		let expected_collateral_auction_size = Self::expected_collateral_auction_size(currency_id);
		let max_auctions_count: Balance = T::MaxAuctionsCount::get().into();
		let lots_count = if !splited
			|| max_auctions_count.is_zero()
			|| expected_collateral_auction_size.is_zero()
			|| amount <= expected_collateral_auction_size
		{
			One::one()
		} else {
			let mut count = amount
				.checked_div(expected_collateral_auction_size)
				.expect("collateral auction maximum size is not zero; qed");

			let remainder = amount
				.checked_rem(expected_collateral_auction_size)
				.expect("collateral auction maximum size is not zero; qed");
			if !remainder.is_zero() {
				count = count.saturating_add(One::one());
			}
			sp_std::cmp::min(count, max_auctions_count)
		};
		let average_amount_per_lot = amount.checked_div(lots_count).expect("lots count is at least 1; qed");
		let average_target_per_lot = target.checked_div(lots_count).expect("lots count is at least 1; qed");
		let mut created_lots: Balance = Zero::zero();

		while !unhandled_collateral_amount.is_zero() {
			created_lots = created_lots.saturating_add(One::one());
			let (lot_collateral_amount, lot_target) = if created_lots == lots_count {
				// the last lot may be have some remnant than average
				(unhandled_collateral_amount, unhandled_target)
			} else {
				(average_amount_per_lot, average_target_per_lot)
			};

			T::AuctionManagerHandler::new_collateral_auction(
				&refund_receiver,
				currency_id,
				lot_collateral_amount,
				lot_target,
			)?;

			unhandled_collateral_amount = unhandled_collateral_amount.saturating_sub(lot_collateral_amount);
			unhandled_target = unhandled_target.saturating_sub(lot_target);
		}
		Ok(())
	}

	fn remove_liquidity_for_lp_collateral(
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		let (currency_id_0, currency_id_1) = lp_currency_id
			.split_dex_share_currency_id()
			.ok_or(Error::<T>::NotDexShare)?;
		T::DEX::remove_liquidity(
			&Self::account_id(),
			currency_id_0,
			currency_id_1,
			amount,
			Zero::zero(),
			Zero::zero(),
		)
	}

	fn max_auction() -> u32 {
		T::MaxAuctionsCount::get()
	}

}

#[cfg(feature = "std")]
impl GenesisConfig {
	/// Direct implementation of `GenesisBuild::build_storage`.
	///
	/// Kept in order not to break dependency.
	pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
		<Self as GenesisBuild<T>>::build_storage(self)
	}

	/// Direct implementation of `GenesisBuild::assimilate_storage`.
	///
	/// Kept in order not to break dependency.
	pub fn assimilate_storage<T: Config>(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
		<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}
