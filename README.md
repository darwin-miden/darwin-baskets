# darwin-baskets

Versioned basket manifests for [Darwin Protocol](https://github.com/darwin-miden) and a Rust loader with validation.

A manifest describes the composition, rebalancing threshold, and fee parameters of a single Darwin basket. Manifests live under `manifests/` as TOML files; the Rust loader (`src/`) parses, validates, and exposes them.

## M1 baskets

| Basket | Symbol | Constituents and weights |
|---|---|---|
| Core Crypto | DCC | dWBTC 40 / dETH 40 / dUSDT 20 |
| Aggressive | DAG | dWBTC 50 / dETH 50 |
| Conservative | DCO | dWBTC 10 / dETH 10 / dUSDT 40 / dDAI 40 |

All three baskets share: 5% drift threshold, 0.30% mint fee, 0.30% redeem fee, 1.00%/year management fee streamed linearly per block.

See [`darwin-docs/m1-architecture-spec.md`](https://github.com/darwin-miden/darwin-docs/blob/main/docs/m1-architecture-spec.md) §6 for the full specification.

## Usage

```rust
use darwin_baskets::{core_crypto, all_m1};

let dcc = core_crypto();
assert_eq!(dcc.symbol, "DCC");
assert_eq!(dcc.total_weight_bps(), 10_000);

for basket in all_m1() {
    basket.validate().expect("basket must be valid");
}
```

## Validation rules

A manifest is valid if and only if:

- The constituents list is non-empty.
- Every `faucet_alias` is unique within the basket.
- Every constituent weight is strictly positive and the sum equals exactly 10000 bps.
- Each fee (mint / redeem / annual management) is at most 10000 bps.
- The drift threshold is strictly positive and at most 5000 bps.
- The symbol is 1–5 ASCII uppercase characters.
- The version is a `major.minor.patch` semver-style tag with digits only.
- `basket_faucet_decimals` is at most 18.

See `src/validation.rs` for the canonical implementation and the precise error variants.

## Adding a basket

1. Create `manifests/<name>.toml` following the schema below.
2. Add a loader function in `src/lib.rs` that calls `load_bundled(include_str!(..))`.
3. Add tests in `tests/manifests.rs`.

## Schema

```toml
[basket]
name = "..."
symbol = "..."        # 1-5 uppercase ASCII
version = "1.0.0"     # semver-style
basket_faucet_decimals = 8

[[basket.constituents]]
faucet_alias = "darwin-..."
target_weight_bps = 4000     # 1..=10000, sum across constituents must equal 10000
pragma_pair = "..."          # Pragma feed identifier, e.g. "ETH/USD"

[basket.rebalancing]
drift_threshold_bps = 500    # 1..=5000

[basket.fees]
mint_fee_bps = 30
redeem_fee_bps = 30
management_fee_bps_year = 100
```

## License

MIT.
