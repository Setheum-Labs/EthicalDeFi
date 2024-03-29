// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of Ethical DeFi.
//
// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)

//! Unit tests for the loans module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn debits_key() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 0);
		assert_ok!(LoansModule::adjust_position(&ALICE, SERP, 200, 200));
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 200);
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 200);
		assert_ok!(LoansModule::adjust_position(&ALICE, SERP, -100, -100));
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 100);
	});
}

#[test]
fn check_update_loan_underflow_work() {
	ExtBuilder::default().build().execute_with(|| {
		// collateral underflow
		assert_noop!(
			LoansModule::update_loan(&ALICE, SERP, -100, 0),
			ArithmeticError::Underflow,
		);

		// debit underflow
		assert_noop!(
			LoansModule::update_loan(&ALICE, SERP, 0, -100),
			ArithmeticError::Underflow,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 1000);

		// balance too low
		assert_noop!(
			LoansModule::adjust_position(&ALICE, SERP, 2000, 0),
			orml_tokens::Error::<Runtime>::BalanceTooLow
		);

		// mock can't pass liquidation ratio check
		assert_noop!(
			LoansModule::adjust_position(&ALICE, DNAR, 500, 0),
			sp_runtime::DispatchError::Other("mock below liquidation ratio error")
		);

		// mock can't pass required ratio check
		assert_noop!(
			LoansModule::adjust_position(&ALICE, DNAR, 500, 1),
			sp_runtime::DispatchError::Other("mock below required collateral ratio error")
		);

		// mock exceed debit value cap
		assert_noop!(
			LoansModule::adjust_position(&ALICE, SERP, 1000, 1000),
			sp_runtime::DispatchError::Other("mock exceed debit value cap error")
		);

		// failed because ED of collateral
		assert_noop!(
			LoansModule::adjust_position(&ALICE, SERP, 99, 0),
			orml_tokens::Error::<Runtime>::ExistentialDeposit,
		);

		assert_eq!(Currencies::free_balance(SERP, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 0);
		assert_eq!(LoansModule::total_positions(SERP).debit, 0);
		assert_eq!(LoansModule::total_positions(SERP).collateral, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 0);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);

		// success
		assert_ok!(LoansModule::adjust_position(&ALICE, SERP, 500, 300));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 500);
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 500);
		assert_eq!(LoansModule::total_positions(SERP).debit, 300);
		assert_eq!(LoansModule::total_positions(SERP).collateral, 500);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 300);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 500);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 150);
		System::assert_has_event(Event::LoansModule(crate::Event::PositionUpdated {
			owner: ALICE,
			collateral_type: SERP,
			collateral_adjustment: 500,
			debit_adjustment: 300,
		}));

		// collateral_adjustment is negatives
		assert_eq!(Currencies::total_balance(SERP, &LoansModule::account_id()), 500);
		assert_ok!(LoansModule::adjust_position(&ALICE, SERP, -500, 0));
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 0);
	});
}

#[test]
fn update_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 1000);
		assert_eq!(LoansModule::total_positions(SERP).debit, 0);
		assert_eq!(LoansModule::total_positions(SERP).collateral, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 0);
		assert!(!<Positions<Runtime>>::contains_key(SERP, &ALICE));

		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(LoansModule::update_loan(&ALICE, SERP, 3000, 2000));

		// just update records
		assert_eq!(LoansModule::total_positions(SERP).debit, 2000);
		assert_eq!(LoansModule::total_positions(SERP).collateral, 3000);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 2000);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 3000);

		// increase ref count when open new position
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);

		// DNAR not manipulate balance
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 1000);

		// should remove position storage if zero
		assert!(<Positions<Runtime>>::contains_key(SERP, &ALICE));
		assert_ok!(LoansModule::update_loan(&ALICE, SERP, -3000, -2000));
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 0);
		assert!(!<Positions<Runtime>>::contains_key(SERP, &ALICE));

		// decrease ref count after remove position
		let alice_ref_count_2 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_2, alice_ref_count_1 - 1);
	});
}

#[test]
fn transfer_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(LoansModule::update_loan(&ALICE, SERP, 400, 500));
		assert_ok!(LoansModule::update_loan(&BOB, SERP, 100, 600));
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 400);
		assert_eq!(LoansModule::positions(SERP, &BOB).debit, 600);
		assert_eq!(LoansModule::positions(SERP, &BOB).collateral, 100);

		assert_ok!(LoansModule::transfer_loan(&ALICE, &BOB, SERP));
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 0);
		assert_eq!(LoansModule::positions(SERP, &BOB).debit, 1100);
		assert_eq!(LoansModule::positions(SERP, &BOB).collateral, 500);
		System::assert_last_event(Event::LoansModule(crate::Event::TransferLoan {
			from: ALICE,
			to: BOB,
			currency_id: SERP,
		}));
	});
}

#[test]
fn confiscate_collateral_and_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(LoansModule::update_loan(&BOB, SERP, 5000, 1000));
		assert_eq!(Currencies::free_balance(SERP, &LoansModule::account_id()), 0);

		// have no sufficient balance
		assert!(!LoansModule::confiscate_collateral_and_debit(&BOB, SERP, 5000, 1000).is_ok(),);

		assert_ok!(LoansModule::adjust_position(&ALICE, SERP, 500, 300));
		assert_eq!(CDPTreasuryModule::get_total_collaterals(SERP), 0);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 300);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 500);

		assert_ok!(LoansModule::confiscate_collateral_and_debit(&ALICE, SERP, 300, 200));
		assert_eq!(CDPTreasuryModule::get_total_collaterals(SERP), 300);
		assert_eq!(CDPTreasuryModule::debit_pool(), 100);
		assert_eq!(LoansModule::positions(SERP, &ALICE).debit, 100);
		assert_eq!(LoansModule::positions(SERP, &ALICE).collateral, 200);
		System::assert_last_event(Event::LoansModule(crate::Event::ConfiscateCollateralAndDebit {
			owner: ALICE,
			collateral_type: SERP,
			confiscated_collateral_amount: 300,
			deduct_debit_amount: 200,
		}));
	});
}

// #[test]
// fn loan_updated_updated_when_adjust_collateral() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(DNAR_SHARES.with(|v| *v.borrow().get(&BOB).unwrap_or(&0)), 0);

// 		assert_ok!(LoansModule::update_loan(&BOB, DNAR, 1000, 0));
// 		assert_eq!(DNAR_SHARES.with(|v| *v.borrow().get(&BOB).unwrap_or(&0)), 1000);

// 		assert_ok!(LoansModule::update_loan(&BOB, DNAR, 0, 200));
// 		assert_eq!(DNAR_SHARES.with(|v| *v.borrow().get(&BOB).unwrap_or(&0)), 1000);

// 		assert_ok!(LoansModule::update_loan(&BOB, DNAR, -800, 500));
// 		assert_eq!(DNAR_SHARES.with(|v| *v.borrow().get(&BOB).unwrap_or(&0)), 200);
// 	});
// }
