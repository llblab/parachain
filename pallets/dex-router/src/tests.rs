//! Unit tests for the DEX router pallet.

use crate::traits::*;
use frame::prelude::*;

#[test]
fn router_fee_calculation_logic() {
  // Test router fee calculation logic
  let amount_in = 1000u128;
  let router_fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  let expected_fee = router_fee_rate.mul_floor(amount_in);
  assert_eq!(expected_fee, 3u128); // 0.3% of 1000 = 3

  let amount_after_fee = amount_in.checked_sub(expected_fee).unwrap();
  assert_eq!(amount_after_fee, 997u128);
}

#[test]
fn large_amount_fee_calculation() {
  // Test with larger amounts
  let amount_in = 1_000_000u128;
  let router_fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  let expected_fee = router_fee_rate.mul_floor(amount_in);
  assert_eq!(expected_fee, 3000u128); // 0.3% of 1,000,000 = 3,000

  let amount_after_fee = amount_in.checked_sub(expected_fee).unwrap();
  assert_eq!(amount_after_fee, 997_000u128);
}

#[test]
fn zero_amount_fee_calculation() {
  // Test with zero amount
  let amount_in = 0u128;
  let router_fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  let expected_fee = router_fee_rate.mul_floor(amount_in);
  assert_eq!(expected_fee, 0u128);

  let amount_after_fee = amount_in.checked_sub(expected_fee).unwrap();
  assert_eq!(amount_after_fee, 0u128);
}

#[test]
fn trait_bounds_compile() {
  // Test that the trait bounds compile correctly
  fn test_balance_traits<T>()
  where
    T: frame::prelude::Parameter
      + frame::prelude::Member
      + Copy
      + Default
      + polkadot_sdk::sp_runtime::traits::Zero
      + polkadot_sdk::sp_runtime::traits::AtLeast32BitUnsigned
      + polkadot_sdk::sp_runtime::traits::Saturating
      + polkadot_sdk::sp_runtime::traits::CheckedSub
      + PartialOrd,
  {
    let _zero = T::zero();
    let _default = T::default();
  }

  // This should compile without errors
  test_balance_traits::<u128>();
  test_balance_traits::<u64>();
  test_balance_traits::<u32>();
}

#[test]
fn checked_arithmetic_safety() {
  // Test checked arithmetic operations
  let max_value = u32::MAX;
  let router_fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  // This should not panic
  let fee = router_fee_rate.mul_floor(max_value);
  let remaining = max_value.checked_sub(fee);

  assert!(remaining.is_some());
  assert!(remaining.unwrap() < max_value);
}

#[test]
fn overflow_protection() {
  // Test overflow protection in fee calculation
  let amount = u128::MAX;
  let router_fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  // This should not panic due to overflow
  let fee = router_fee_rate.mul_floor(amount);
  let remaining = amount.checked_sub(fee);

  assert!(remaining.is_some());
  assert_eq!(remaining.unwrap(), amount - fee);
}

#[test]
fn weight_info_trait() {
  // Test that WeightInfo trait is properly defined
  use crate::WeightInfo;

  let weight = <() as WeightInfo>::swap_exact_tokens_for_tokens();
  assert_eq!(weight.ref_time(), 10_000);
  assert_eq!(weight.proof_size(), 0);
}

#[test]
fn routing_strategy_trait_exists() {
  // Test that RoutingStrategy trait exists and can be used
  use crate::traits::RoutingStrategy;

  // Test that BestPriceStrategy exists
  let strategy = BestPriceStrategy;

  // Test with empty quotes
  let empty_quotes: Vec<(AMMType, u128)> = vec![];
  let result = strategy.select_best_amm(empty_quotes, &(), &());
  assert!(result.is_none());

  // Test with single quote
  let single_quote = vec![(AMMType::XYK, 100u128)];
  let result = strategy.select_best_amm(single_quote, &(), &());
  assert_eq!(result, Some(AMMType::XYK));

  // Test with multiple quotes - should select highest
  let multiple_quotes = vec![(AMMType::XYK, 100u128), (AMMType::TBC, 200u128)];
  let result = strategy.select_best_amm(multiple_quotes, &(), &());
  assert_eq!(result, Some(AMMType::TBC));
}

#[test]
fn amm_trait_name_method() {
  // Test that AMM trait name method works
  use crate::traits::AMM;

  // Create a mock AMM implementation
  struct MockAMM;

  impl AMM<(), u128, u64> for MockAMM {
    type Error = &'static str;

    fn can_handle_pair(&self, _asset_in: &(), _asset_out: &()) -> bool {
      true
    }

    fn quote_price(&self, _asset_in: &(), _asset_out: &(), _amount_in: u128) -> Option<u128> {
      Some(_amount_in)
    }

    fn execute_swap(
      &self,
      _who: &u64,
      _asset_in: (),
      _asset_out: (),
      _amount_in: u128,
      _min_amount_out: u128,
    ) -> Result<u128, Self::Error> {
      Ok(_amount_in)
    }

    fn name(&self) -> &'static str {
      "MockAMM"
    }
  }

  let mock_amm = MockAMM;
  assert_eq!(mock_amm.name(), "MockAMM");
  assert!(mock_amm.can_handle_pair(&(), &()));
  assert_eq!(mock_amm.quote_price(&(), &(), 100), Some(100));
  assert_eq!(mock_amm.execute_swap(&1u64, (), (), 100, 90), Ok(100));
}

#[test]
fn fee_collector_trait_exists() {
  // Test that FeeCollector trait exists
  use crate::traits::FeeCollector;

  // Create a mock fee collector
  struct MockFeeCollector;

  impl FeeCollector<(), u128, u64> for MockFeeCollector {
    fn collect_fee(
      &self,
      _from: &u64,
      _asset: &(),
      _amount: u128,
    ) -> frame::prelude::DispatchResult {
      Ok(())
    }
  }

  let mock_collector = MockFeeCollector;
  assert_eq!(mock_collector.collect_fee(&1u64, &(), 100), Ok(()));
}

#[test]
fn permill_operations() {
  // Test Permill operations used in fee calculations
  let fee_rate = Permill::from_rational(3u32, 1000u32); // 0.3%

  // Test different amounts
  assert_eq!(fee_rate.mul_floor(100u128), 0u128);
  assert_eq!(fee_rate.mul_floor(1000u128), 3u128);
  assert_eq!(fee_rate.mul_floor(10000u128), 30u128);

  // Test that it's actually 0.3%
  let fee_parts = fee_rate.deconstruct();
  assert_eq!(fee_parts, 3000u32); // 0.3% = 3000 parts per million
}
