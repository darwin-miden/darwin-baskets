//! Print the bundled `state/testnet.toml` as a human-readable summary.
//!
//! Same data the Miden grant reviewer can see on
//! `testnet.midenscan.com` — but laid out in a single screen so the
//! state is easy to audit alongside the deployment recipe in
//! `darwin-infra/scripts/deploy-testnet.sh`.
//!
//! Usage:
//!     cargo run --bin testnet_inventory

use darwin_baskets::load_testnet;

fn main() {
    let s = load_testnet();

    println!("Darwin Protocol — Miden testnet deployment inventory");
    println!("=====================================================");
    println!(
        "Network: {} (first deployment at block {})",
        s.network.name, s.network.chain_block_height_at_first_deployment
    );

    println!();
    println!("Asset faucets ({}):", s.asset_faucets.len());
    for (key, f) in &s.asset_faucets {
        println!(
            "  {:<8} {:<8} decimals={:<2} max_supply={:<14} {}",
            key, f.symbol, f.decimals, f.max_supply, f.account_id
        );
    }

    println!();
    println!("Basket-token faucets ({}):", s.basket_token_faucets.len());
    for (key, f) in &s.basket_token_faucets {
        println!(
            "  {:<5} decimals={:<2} max_supply={:<14} {}",
            key, f.decimals, f.max_supply, f.account_id
        );
    }

    println!();
    println!("Protocol accounts ({}):", s.protocol_accounts.len());
    for (key, pa) in &s.protocol_accounts {
        println!(
            "  {:<12} basket={:<5} {}",
            key, pa.basket, pa.account_id
        );
    }

    println!();
    println!("Team wallet:");
    println!("  {}", s.test_wallet.account_id);

    if let Some(uw) = &s.user_wallet {
        println!();
        println!("User wallet (Flow A simulation):");
        println!("  {}", uw.account_id);
        println!("  first P2ID tx: {}", uw.first_p2id_tx);
        println!("  first P2ID asset: {}", uw.first_p2id_asset);
    }

    if !s.pool_funding.is_empty() {
        println!();
        println!("Pool funding mints ({}):", s.pool_funding.len());
        for pf in &s.pool_funding {
            println!(
                "  {:<5} {:<8} amount={:<10} tx={}",
                pf.basket, pf.asset, pf.amount, pf.mint_tx
            );
        }
    }

    if !s.user_wallet_mints.is_empty() {
        println!();
        println!("User wallet mints ({}):", s.user_wallet_mints.len());
        for um in &s.user_wallet_mints {
            let note = um.note_id.as_deref().unwrap_or("(not recorded)");
            println!(
                "  {:<8} amount={:<10} tx={}",
                um.asset, um.amount, um.mint_tx
            );
            println!("           note={note}");
        }
    }

    println!();
    println!(
        "Total: {} on-chain accounts, {} pool-funding mints, {} user-wallet mints.",
        s.asset_faucets.len()
            + s.basket_token_faucets.len()
            + s.protocol_accounts.len()
            + 1
            + s.user_wallet.is_some() as usize,
        s.pool_funding.len(),
        s.user_wallet_mints.len(),
    );
}
