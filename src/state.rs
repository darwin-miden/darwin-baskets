//! Typed loader for `state/testnet.toml` — the authoritative on-chain
//! inventory of every Darwin account currently live on the public
//! Miden testnet.
//!
//! The TOML format is hand-maintained when deployments land; this
//! module provides a typed view for tooling and CI checks (e.g. the
//! `testnet_inventory` binary prints a human-readable summary, and the
//! `cross_state_tests` module asserts that every basket constituent
//! has a matching deployed asset faucet).

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct TestnetDeployment {
    pub network: NetworkInfo,
    #[serde(default)]
    pub asset_faucets: BTreeMap<String, FaucetEntry>,
    #[serde(default)]
    pub basket_token_faucets: BTreeMap<String, FaucetEntry>,
    #[serde(default)]
    pub protocol_accounts: BTreeMap<String, ProtocolAccountEntry>,
    pub test_wallet: WalletEntry,
    #[serde(default)]
    pub user_wallet: Option<UserWalletEntry>,
    #[serde(default, rename = "pool_funding")]
    pub pool_funding: Vec<PoolFundingEntry>,
    #[serde(default, rename = "user_wallet_mints")]
    pub user_wallet_mints: Vec<UserMintEntry>,
    #[serde(default, rename = "user_deposits")]
    pub user_deposits: Vec<UserDepositEntry>,
    #[serde(default, rename = "atomic_flow_a_notes")]
    pub atomic_flow_a_notes: Vec<AtomicFlowANoteEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtomicFlowANoteEntry {
    pub note_id: String,
    #[serde(default)]
    pub note_script_root: Option<String>,
    pub sender_wallet: String,
    pub target_controller: String,
    pub asset: String,
    pub asset_faucet_id: String,
    pub amount: u64,
    pub submit_tx: String,
    pub submission_block: u64,
    #[serde(default)]
    pub note_inputs: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserDepositEntry {
    pub basket: String,
    pub controller_id: String,
    pub asset: String,
    pub amount: u64,
    pub send_tx: String,
    pub note_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub chain_block_height_at_first_deployment: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaucetEntry {
    pub account_id: String,
    pub symbol: String,
    pub decimals: u8,
    pub max_supply: u64,
    pub deployed_by_tx: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProtocolAccountEntry {
    pub account_id: String,
    pub basket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletEntry {
    pub account_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserWalletEntry {
    pub account_id: String,
    pub first_p2id_tx: String,
    pub first_p2id_asset: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PoolFundingEntry {
    pub basket: String,
    pub asset: String,
    pub amount: u64,
    pub mint_tx: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserMintEntry {
    pub asset: String,
    pub faucet_id: String,
    pub amount: u64,
    pub mint_tx: String,
    #[serde(default)]
    pub note_id: Option<String>,
}

/// Bundled snapshot of the testnet state — included at compile time so
/// downstream crates and the `testnet_inventory` binary don't need a
/// runtime file path.
pub const TESTNET_TOML: &str = include_str!("../state/testnet.toml");

/// Parses the bundled TOML snapshot. Used by tooling and by the
/// integration tests below.
pub fn load_testnet() -> TestnetDeployment {
    toml::from_str(TESTNET_TOML).expect("bundled testnet.toml must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_testnet_loads() {
        let state = load_testnet();
        assert_eq!(state.network.name, "miden-testnet");
        assert!(state.network.chain_block_height_at_first_deployment > 0);
    }

    #[test]
    fn every_asset_faucet_has_distinct_account_id() {
        let state = load_testnet();
        let mut ids = std::collections::HashSet::new();
        for (name, f) in &state.asset_faucets {
            assert!(
                ids.insert(f.account_id.clone()),
                "{} reuses account id {}",
                name,
                f.account_id
            );
        }
    }

    #[test]
    fn every_protocol_account_corresponds_to_a_known_basket() {
        let state = load_testnet();
        // PROTO is a deployment-tooling prototype, not an M1 basket.
        let valid_baskets: Vec<&'static str> = vec!["DCC", "DAG", "DCO", "PROTO"];
        for (k, pa) in &state.protocol_accounts {
            assert!(
                valid_baskets.contains(&pa.basket.as_str()),
                "protocol account {} references unknown basket {}",
                k,
                pa.basket
            );
        }
    }

    #[test]
    fn user_wallet_is_recorded_with_seed_assets() {
        let state = load_testnet();
        let uw = state.user_wallet.as_ref().expect("user wallet recorded");
        assert!(uw.account_id.starts_with("0x"));
        assert!(uw.first_p2id_tx.starts_with("0x"));
        // The user must hold at least one Darwin-asset mint to drive
        // the Flow A simulation.
        assert!(
            !state.user_wallet_mints.is_empty(),
            "user wallet has no Darwin-asset mints"
        );
    }

    #[test]
    fn user_wallet_holds_every_m1_constituent() {
        let state = load_testnet();
        // Convention: user mint assets are listed as "dETH"/"dWBTC"/etc;
        // the M1 baskets cover ETH, WBTC, USDT, DAI. The wallet should
        // hold one entry per constituent for the Flow A demo to apply
        // to every basket.
        let want = ["dETH", "dWBTC", "dUSDT", "dDAI"];
        for w in want {
            assert!(
                state.user_wallet_mints.iter().any(|m| m.asset == w),
                "user wallet missing {} mint",
                w
            );
        }
    }

    #[test]
    fn user_wallet_received_all_three_basket_tokens() {
        let state = load_testnet();
        // Deliverable #2 of the M1 grant: basket tokens mintable
        // natively on Miden. The user wallet must have received a
        // mint from each of DCC, DAG, DCO.
        for symbol in ["DCC", "DAG", "DCO"] {
            assert!(
                state.user_wallet_mints.iter().any(|m| m.asset == symbol),
                "user wallet missing {} mint",
                symbol
            );
        }
    }

    #[test]
    fn user_deposit_table_covers_every_basket() {
        let state = load_testnet();
        // The deposit half of Flow A: at least one user-side P2ID
        // transfer to each basket's controller.
        for symbol in ["DCC", "DAG", "DCO"] {
            assert!(
                state.user_deposits.iter().any(|d| d.basket == symbol),
                "no user deposit recorded for {}",
                symbol
            );
        }
    }

    #[test]
    fn user_deposits_target_known_protocol_accounts() {
        let state = load_testnet();
        let known: std::collections::HashSet<String> = state
            .protocol_accounts
            .values()
            .map(|pa| pa.account_id.clone())
            .collect();
        for d in &state.user_deposits {
            assert!(
                known.contains(&d.controller_id),
                "deposit {} -> {} targets unknown controller",
                d.basket,
                d.controller_id
            );
        }
    }

    #[test]
    fn pool_funding_targets_only_declared_baskets() {
        let state = load_testnet();
        for pf in &state.pool_funding {
            assert!(
                ["DCC", "DAG", "DCO"].contains(&pf.basket.as_str()),
                "pool funding targets unknown basket {}",
                pf.basket
            );
            assert!(pf.amount > 0, "pool funding amount is zero");
            assert!(pf.mint_tx.starts_with("0x"));
        }
    }

    #[test]
    fn every_manifest_constituent_has_a_deployed_asset_faucet() {
        let state = load_testnet();
        let alias_to_symbol = [
            ("darwin-eth", "DETH"),
            ("darwin-wbtc", "DWBTC"),
            ("darwin-usdt", "DUSDT"),
            ("darwin-dai", "DDAI"),
        ];
        for alias in super::super::all_m1_faucet_aliases() {
            let symbol = alias_to_symbol
                .iter()
                .find(|(a, _)| *a == alias)
                .map(|(_, s)| *s)
                .unwrap_or_else(|| panic!("no known faucet for alias {alias}"));
            assert!(
                state
                    .asset_faucets
                    .values()
                    .any(|f| f.symbol == symbol),
                "manifest alias {alias} ({symbol}) has no deployed faucet",
            );
        }
    }
}
