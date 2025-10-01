use anchor_lang::AccountDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use temple::state::global_stats::GlobalStats;
use tokio::time;

pub struct IndexerFetcher {
    rpc_client: RpcClient,
    program_id: Pubkey,
}

impl IndexerFetcher {
    pub fn new(rpc_url: &str, program_id: Pubkey) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Self {
            rpc_client,
            program_id,
        }
    }

    pub async fn start_polling(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            self.fetch_global_stats().await?;
        }
    }

    async fn fetch_global_stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (global_stats_pda, _) =
            Pubkey::find_program_address(&[b"global_stats_v1"], &self.program_id);

        match self.rpc_client.get_account(&global_stats_pda) {
            Ok(account) => match GlobalStats::try_deserialize(&mut &account.data[..]) {
                Ok(global_stats) => {
                    println!("Fetched GlobalStats successfully:");
                    println!("  Total Merit: {}", global_stats.total_merit);
                    println!(
                        "  Total Incense Points: {}",
                        global_stats.total_incense_points
                    );
                    println!(
                        "  Total Donations (SOL): {:.9}",
                        global_stats.total_donations_sol()
                    );
                    println!("  Total Users: {}", global_stats.total_users);
                    println!("  Total Wishes: {}", global_stats.total_wishes);
                    println!("  Updated At: {}", global_stats.updated_at);
                }
                Err(e) => {
                    eprintln!("Failed to deserialize GlobalStats: {:?}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch GlobalStats account: {:?}", e);
            }
        }

        Ok(())
    }
}
