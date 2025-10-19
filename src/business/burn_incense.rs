use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::{MySql, Pool};

use crate::{
    domain::IncenseRulesConfig,
    infra::write_config_to_db,
    utils::{account_disc, decode_anchor_account},
};

pub async fn fetch_burn_incense_type_once(
    rpc: &RpcClient,
    program_id: Pubkey,
    pool: &Pool<MySql>,
) -> anyhow::Result<()> {
    let (pda, _bump) = Pubkey::find_program_address(&[b"incense_rules_config"], &program_id);
    let acc_opt = rpc
        .get_account_with_commitment(&pda, CommitmentConfig::finalized())
        .await?
        .value;

    let Some(acc) = acc_opt else {
        println!("IncenseRulesConfig PDA not found: {}", pda);
        return Ok(());
    };

    let disc = account_disc("IncenseRulesConfig");

    let config: IncenseRulesConfig = decode_anchor_account(&acc.data, &disc)?;
    println!("IncenseRulesConfig: {:?}", config.rules);

    let names = [
        "Clear Incense",
        "Sandalwood",
        "Ambergris Incense",
        "Supreme Spirit Incense",
        "Secret Brew Incense",
        "Celestial Incense",
    ];
    let admin = config.admin;

    let rules = config.rules;
    for (i, v) in rules.iter().enumerate() {
        let name = names[i];
        write_config_to_db(pool, name, &admin.to_string(), v).await?;
    }

    Ok(())
}
