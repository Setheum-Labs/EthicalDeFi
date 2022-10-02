// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of Zims.
//
// Copyright (C) 2019-2022 Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)

//! Autogenerated weights for serp_treasury
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-02-26, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=serp_treasury
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./lib-serml/serp-treasury/src/weights.rs
// --template=./templates/setheum-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for cdp_treasury.
pub trait WeightInfo {
	fn on_initialize(c: u32) -> Weight;
	fn set_stable_currency_inflation_rate() -> Weight;
	fn force_serpdown() -> Weight;
}

/// Weights for serp_treasury using the Setheum node and recommended hardware.
pub struct SetheumWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SetheumWeight<T> {
	fn on_initialize(c: u32) -> Weight {
		(243_267_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(30 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_stable_currency_inflation_rate() -> Weight {
		(20_458_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn force_serpdown() -> Weight {
		(33_360_000 as Weight)
			.saturating_add((23_139_000 as Weight).saturating_mul(1 as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(1 as Weight)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn on_initialize(c: u32) -> Weight {
		(243_267_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(30 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_stable_currency_inflation_rate() -> Weight {
		(20_458_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn force_serpdown() -> Weight {
		(33_360_000 as Weight)
			.saturating_add((23_139_000 as Weight).saturating_mul(1 as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(1 as Weight)))
	}
}
