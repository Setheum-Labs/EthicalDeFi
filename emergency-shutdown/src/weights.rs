// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of Ethical DeFi.
//
// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)


//! Autogenerated weights for emergency_shutdown
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-02-26, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/setheum-node
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=emergency_shutdown
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./lib-serml/emergency-shutdown/src/weights.rs
// --template=./templates/module-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for emergency_shutdown.
pub trait WeightInfo {
	fn emergency_shutdown(c: u32, ) -> Weight;
	fn open_collateral_refund() -> Weight;
	fn refund_collaterals(c: u32, ) -> Weight;
}

/// Weights for emergency_shutdown using the Setheum node and recommended hardware.
pub struct SetheumWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SetheumWeight<T> {
	fn emergency_shutdown(c: u32, ) -> Weight {
		(232_768_000 as Weight)
			// Standard Error: 565_000
			.saturating_add((20_539_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(60 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(c as Weight)))
	}
	fn open_collateral_refund() -> Weight {
		(62_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn refund_collaterals(c: u32, ) -> Weight {
		(122_271_000 as Weight)
			// Standard Error: 215_000
			.saturating_add((34_100_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn emergency_shutdown(c: u32, ) -> Weight {
		(232_768_000 as Weight)
			// Standard Error: 565_000
			.saturating_add((20_539_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(60 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes((3 as Weight).saturating_mul(c as Weight)))
	}
	fn open_collateral_refund() -> Weight {
		(62_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(17 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn refund_collaterals(c: u32, ) -> Weight {
		(122_271_000 as Weight)
			// Standard Error: 215_000
			.saturating_add((34_100_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
	}
}
