use darwin_baskets::{
    aggressive, all_m1, conservative, core_crypto, BasketManifest, ValidationError,
};

#[test]
fn core_crypto_loads_and_validates() {
    let m = core_crypto();
    assert_eq!(m.symbol, "DCC");
    assert_eq!(m.constituents.len(), 3);
    assert_eq!(m.total_weight_bps(), 10_000);
}

#[test]
fn aggressive_loads_and_validates() {
    let m = aggressive();
    assert_eq!(m.symbol, "DAG");
    assert_eq!(m.constituents.len(), 2);
    assert_eq!(m.total_weight_bps(), 10_000);
}

#[test]
fn conservative_loads_and_validates() {
    let m = conservative();
    assert_eq!(m.symbol, "DCO");
    assert_eq!(m.constituents.len(), 4);
    assert_eq!(m.total_weight_bps(), 10_000);
}

#[test]
fn all_m1_baskets_have_unique_symbols() {
    let baskets = all_m1();
    let mut symbols: Vec<_> = baskets.iter().map(|b| b.symbol.clone()).collect();
    symbols.sort();
    let original_len = symbols.len();
    symbols.dedup();
    assert_eq!(symbols.len(), original_len, "basket symbols collide");
}

#[test]
fn all_m1_baskets_use_known_pragma_pairs() {
    let known = ["BTC/USD", "ETH/USD", "WBTC/USD", "USDT/USD", "DAI/USD"];
    for basket in all_m1() {
        for c in &basket.constituents {
            assert!(
                known.contains(&c.pragma_pair.as_str()),
                "basket {} uses unknown Pragma pair {}",
                basket.symbol,
                c.pragma_pair
            );
        }
    }
}

#[test]
fn rejects_weight_sum_mismatch() {
    let toml = r#"
        [basket]
        name = "Broken"
        symbol = "BRK"
        version = "1.0.0"
        basket_faucet_decimals = 8

        [[basket.constituents]]
        faucet_alias = "darwin-eth"
        target_weight_bps = 3000
        pragma_pair = "ETH/USD"

        [[basket.constituents]]
        faucet_alias = "darwin-wbtc"
        target_weight_bps = 3000
        pragma_pair = "WBTC/USD"

        [basket.rebalancing]
        drift_threshold_bps = 500

        [basket.fees]
        mint_fee_bps = 30
        redeem_fee_bps = 30
        management_fee_bps_year = 100
    "#;
    let m = BasketManifest::from_toml_str(toml).expect("parses");
    assert!(matches!(
        m.validate(),
        Err(ValidationError::WeightSumMismatch { sum: 6000, expected: 10_000 })
    ));
}

#[test]
fn rejects_duplicate_constituent() {
    let toml = r#"
        [basket]
        name = "Broken"
        symbol = "BRK"
        version = "1.0.0"
        basket_faucet_decimals = 8

        [[basket.constituents]]
        faucet_alias = "darwin-eth"
        target_weight_bps = 5000
        pragma_pair = "ETH/USD"

        [[basket.constituents]]
        faucet_alias = "darwin-eth"
        target_weight_bps = 5000
        pragma_pair = "ETH/USD"

        [basket.rebalancing]
        drift_threshold_bps = 500

        [basket.fees]
        mint_fee_bps = 30
        redeem_fee_bps = 30
        management_fee_bps_year = 100
    "#;
    let m = BasketManifest::from_toml_str(toml).expect("parses");
    assert!(matches!(
        m.validate(),
        Err(ValidationError::DuplicateConstituent(_))
    ));
}

#[test]
fn rejects_invalid_symbol() {
    let toml = r#"
        [basket]
        name = "Broken"
        symbol = "lowercase"
        version = "1.0.0"
        basket_faucet_decimals = 8

        [[basket.constituents]]
        faucet_alias = "darwin-eth"
        target_weight_bps = 10000
        pragma_pair = "ETH/USD"

        [basket.rebalancing]
        drift_threshold_bps = 500

        [basket.fees]
        mint_fee_bps = 30
        redeem_fee_bps = 30
        management_fee_bps_year = 100
    "#;
    let m = BasketManifest::from_toml_str(toml).expect("parses");
    assert!(matches!(m.validate(), Err(ValidationError::InvalidSymbol(_))));
}

#[test]
fn rejects_zero_weight() {
    let toml = r#"
        [basket]
        name = "Broken"
        symbol = "BRK"
        version = "1.0.0"
        basket_faucet_decimals = 8

        [[basket.constituents]]
        faucet_alias = "darwin-eth"
        target_weight_bps = 0
        pragma_pair = "ETH/USD"

        [[basket.constituents]]
        faucet_alias = "darwin-wbtc"
        target_weight_bps = 10000
        pragma_pair = "WBTC/USD"

        [basket.rebalancing]
        drift_threshold_bps = 500

        [basket.fees]
        mint_fee_bps = 30
        redeem_fee_bps = 30
        management_fee_bps_year = 100
    "#;
    let m = BasketManifest::from_toml_str(toml).expect("parses");
    assert!(matches!(
        m.validate(),
        Err(ValidationError::ZeroWeight { .. })
    ));
}
