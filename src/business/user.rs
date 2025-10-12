use crate::{
    domain::{MedalLevel, UserInfo},
    infra::write_user_info_to_db,
    utils::{account_disc, decode_anchor_account},
};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::{MySql, Pool};

pub async fn fetch_all_user_info(
    rpc: &RpcClient,
    program_id: Pubkey,
    pool: &Pool<MySql>,
) -> anyhow::Result<()> {
    let disc = account_disc("UserInfo");
    let mem = Memcmp::new(0, MemcmpEncodedBytes::Bytes(disc.to_vec()));

    let cfg = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(mem)]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::finalized()),
            min_context_slot: None,
        },
        ..Default::default()
    };
    let accounts = rpc
        .get_program_accounts_with_config(&program_id, cfg)
        .await?;

    for (_pubkey, acc) in accounts {
        let u: UserInfo = decode_anchor_account(&acc.data, &disc)?;
        let re = borsh::to_vec(&u)?;
        assert_eq!(
            re.len(),
            acc.data.len() - 8,
            "local UserInfo layout != on-chain"
        );

        let medel_level = medal_opt_business_code(&u.current_medal_level);
        // let medal_level = medal_opt_business_code(&u.current_medal_level);
        println!("UserInfo current_medal_level :{}", medel_level);

        write_user_info_to_db(pool, u, medel_level).await?;

        println!("write user to db scucess")
    }
    Ok(())
}

fn medal_opt_business_code(m: &MedalLevel) -> u8 {
    match m {
        MedalLevel::None => 0,
        MedalLevel::Bronze => 1,
        MedalLevel::Silver => 2,
        MedalLevel::Gold => 3,
        MedalLevel::Supreme => 4,
    }
}
