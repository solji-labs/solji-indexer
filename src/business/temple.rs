use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::{MySql, Pool};

use crate::{
    domain::Temple,
    infra::write_temple_to_db,
    utils::{account_disc, decode_anchor_account},
};

pub async fn fetch_temple_once(
    rpc: &RpcClient,
    program_id: Pubkey,
    pool: &Pool<MySql>,
) -> anyhow::Result<()> {
    let (pda, _bump) = Pubkey::find_program_address(&[b"temple"], &program_id);
    let acc_opt = rpc
        .get_account_with_commitment(&pda, CommitmentConfig::finalized())
        .await?
        .value;

    let Some(acc) = acc_opt else {
        println!("temple PDA not found: {}", pda);
        return Ok(());
    };

    let disc = account_disc("Temple");

    let temple: Temple = decode_anchor_account(&acc.data, &disc)?;

    println!("temple:\n{}", temple.admin.to_string());

    write_temple_to_db(pool, temple).await?;
    Ok(())
}
