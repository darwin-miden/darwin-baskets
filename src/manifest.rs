use serde::{Deserialize, Serialize};

use crate::validation::ValidationError;

/// Top-level manifest file format. A manifest file contains exactly one
/// `[basket]` section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    pub basket: BasketManifest,
}

/// A single basket's full configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasketManifest {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub basket_faucet_decimals: u8,
    pub constituents: Vec<Constituent>,
    pub rebalancing: BasketRebalancing,
    pub fees: BasketFees,
}

/// One asset in the basket, with its target weight and the Pragma price feed
/// the protocol reads to value it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constituent {
    pub faucet_alias: String,
    pub target_weight_bps: u32,
    pub pragma_pair: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasketRebalancing {
    pub drift_threshold_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasketFees {
    pub mint_fee_bps: u32,
    pub redeem_fee_bps: u32,
    pub management_fee_bps_year: u32,
}

impl BasketManifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        let file: ManifestFile = toml::from_str(s)?;
        Ok(file.basket)
    }

    /// Parse a manifest from a file on disk.
    pub fn from_path(path: &std::path::Path) -> Result<Self, ManifestLoadError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_toml_str(&contents).map_err(ManifestLoadError::Toml)
    }

    /// Run all structural validations on the manifest. Returns the first
    /// failure encountered.
    pub fn validate(&self) -> Result<(), ValidationError> {
        crate::validation::validate(self)
    }

    /// Sum of the target weights, in basis points. A valid basket has
    /// `total_weight_bps() == 10_000`.
    pub fn total_weight_bps(&self) -> u32 {
        self.constituents.iter().map(|c| c.target_weight_bps).sum()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestLoadError {
    #[error("I/O error reading manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(toml::de::Error),
}
