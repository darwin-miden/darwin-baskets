use crate::manifest::BasketManifest;

/// Total weight in basis points that a valid basket must sum to.
pub const TOTAL_WEIGHT_BPS: u32 = 10_000;

/// Maximum allowed fee, in basis points (10000 = 100%). Any individual fee
/// above this is rejected — a sanity bound, well above any realistic value.
pub const MAX_FEE_BPS: u32 = 10_000;

/// Maximum allowed drift threshold, in basis points. A threshold above this
/// would effectively disable rebalancing.
pub const MAX_DRIFT_THRESHOLD_BPS: u32 = 5_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("basket has no constituents")]
    NoConstituents,

    #[error("basket has duplicate constituent {0}")]
    DuplicateConstituent(String),

    #[error("constituent {alias} has weight 0")]
    ZeroWeight { alias: String },

    #[error(
        "constituent weights sum to {sum} bps; must equal {expected} bps"
    )]
    WeightSumMismatch { sum: u32, expected: u32 },

    #[error("fee {field} is {value} bps; must be <= {max} bps")]
    FeeOutOfRange { field: &'static str, value: u32, max: u32 },

    #[error(
        "drift threshold is {value} bps; must be > 0 and <= {max} bps"
    )]
    DriftOutOfRange { value: u32, max: u32 },

    #[error("basket symbol must be 1-5 ASCII uppercase characters; got {0:?}")]
    InvalidSymbol(String),

    #[error("basket version {0:?} is not a recognised semver tag (major.minor.patch)")]
    InvalidVersion(String),

    #[error("basket_faucet_decimals must be between 0 and 18; got {0}")]
    DecimalsOutOfRange(u8),
}

pub(crate) fn validate(m: &BasketManifest) -> Result<(), ValidationError> {
    if m.constituents.is_empty() {
        return Err(ValidationError::NoConstituents);
    }

    let mut seen = std::collections::HashSet::new();
    for c in &m.constituents {
        if !seen.insert(c.faucet_alias.as_str()) {
            return Err(ValidationError::DuplicateConstituent(c.faucet_alias.clone()));
        }
        if c.target_weight_bps == 0 {
            return Err(ValidationError::ZeroWeight {
                alias: c.faucet_alias.clone(),
            });
        }
    }

    let sum = m.total_weight_bps();
    if sum != TOTAL_WEIGHT_BPS {
        return Err(ValidationError::WeightSumMismatch {
            sum,
            expected: TOTAL_WEIGHT_BPS,
        });
    }

    check_fee("mint_fee_bps", m.fees.mint_fee_bps)?;
    check_fee("redeem_fee_bps", m.fees.redeem_fee_bps)?;
    check_fee("management_fee_bps_year", m.fees.management_fee_bps_year)?;

    let drift = m.rebalancing.drift_threshold_bps;
    if drift == 0 || drift > MAX_DRIFT_THRESHOLD_BPS {
        return Err(ValidationError::DriftOutOfRange {
            value: drift,
            max: MAX_DRIFT_THRESHOLD_BPS,
        });
    }

    check_symbol(&m.symbol)?;
    check_version(&m.version)?;

    if m.basket_faucet_decimals > 18 {
        return Err(ValidationError::DecimalsOutOfRange(m.basket_faucet_decimals));
    }

    Ok(())
}

fn check_fee(field: &'static str, value: u32) -> Result<(), ValidationError> {
    if value > MAX_FEE_BPS {
        return Err(ValidationError::FeeOutOfRange {
            field,
            value,
            max: MAX_FEE_BPS,
        });
    }
    Ok(())
}

fn check_symbol(symbol: &str) -> Result<(), ValidationError> {
    let len = symbol.len();
    if !(1..=5).contains(&len) || !symbol.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(ValidationError::InvalidSymbol(symbol.to_string()));
    }
    Ok(())
}

fn check_version(version: &str) -> Result<(), ValidationError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(ValidationError::InvalidVersion(version.to_string()));
    }
    if !parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())) {
        return Err(ValidationError::InvalidVersion(version.to_string()));
    }
    Ok(())
}
