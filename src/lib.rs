//! Darwin Protocol basket manifests.
//!
//! A manifest describes the composition, rebalancing thresholds, and fee
//! parameters of a single Darwin basket. Manifests are stored as TOML files
//! under `manifests/` and loaded at build time or runtime by the protocol,
//! the SDK, and any indexer that needs to interpret on-chain basket state.

mod manifest;
mod validation;

pub use manifest::{BasketFees, BasketManifest, BasketRebalancing, Constituent, ManifestFile};
pub use validation::ValidationError;

/// Returns the manifest of the Core Crypto basket bundled with this crate.
pub fn core_crypto() -> BasketManifest {
    load_bundled(include_str!("../manifests/core-crypto.toml"))
}

/// Returns the manifest of the Aggressive basket bundled with this crate.
pub fn aggressive() -> BasketManifest {
    load_bundled(include_str!("../manifests/aggressive.toml"))
}

/// Returns the manifest of the Conservative basket bundled with this crate.
pub fn conservative() -> BasketManifest {
    load_bundled(include_str!("../manifests/conservative.toml"))
}

/// Returns the three baskets shipped in Milestone 1.
pub fn all_m1() -> [BasketManifest; 3] {
    [core_crypto(), aggressive(), conservative()]
}

/// Lookup a manifest by its on-chain symbol. Returns `None` for any
/// symbol that is not part of the M1 catalogue.
pub fn by_symbol(symbol: &str) -> Option<BasketManifest> {
    match symbol {
        "DCC" => Some(core_crypto()),
        "DAG" => Some(aggressive()),
        "DCO" => Some(conservative()),
        _ => None,
    }
}

/// All faucet aliases referenced by any M1 basket, deduplicated and
/// sorted. Useful for tooling that needs to know which custom asset
/// faucets must be deployed before any basket can be funded.
pub fn all_m1_faucet_aliases() -> Vec<String> {
    let mut aliases: Vec<String> = all_m1()
        .iter()
        .flat_map(|b| b.constituents.iter().map(|c| c.faucet_alias.clone()))
        .collect();
    aliases.sort();
    aliases.dedup();
    aliases
}

fn load_bundled(contents: &str) -> BasketManifest {
    let manifest =
        BasketManifest::from_toml_str(contents).expect("bundled basket manifest must parse");
    manifest
        .validate()
        .expect("bundled basket manifest must pass validation");
    manifest
}
