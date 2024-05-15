use async_trait::async_trait;

use deadpool::managed::{self, Metrics, Pool};

use deadpool::managed::RecycleResult;

use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

use solana_client::nonblocking::pubsub_client::PubsubClient;

pub struct RpcPoolManager {
    pub rpc_type: Option<String>,
}

#[derive(Debug)]
pub enum Error {}

const wss_url: &str = "wss://api.mainnet-beta.solana.com/";

#[async_trait]
impl managed::Manager for RpcPoolManager {
    type Type = RpcClient;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        // println!("Creating new connection {:#?}", self.rpc_type);

        let mut rpc_url: &str =
            "https://din-lb.solanarpc.dev/KsnzZimk2FZ7c4AHPd3EjGLuXjVnRZ5v3X3mgkq";

        // let mut rpc_url = "https://api.solanarpc.dev/rpc/solana/mainnet?token=MjI4fE8yeW0zN0s3T251QnY5V1FMcXF4eGRxdVFNbVlaeUYxYWZXRGJLN0U";

        // let mut rpc_url = "https://rpc.ankr.com/solana/71915acca8127aacb9f83c90556138f82decde6b7a66f5fad32d2e005c26ca8e";

        // let mut rpc_url = "http://api.mainnet-beta.solana.com/";
        // let mut rpc_url = "https://rpc.ankr.com/solana/71915acca8127aacb9f83c90556138f82decde6b7a66f5fad32d2e005c26ca8e";
        // let mut rpc_url = "http://65.108.76.168:8899";
        // let mut rpc_url = "https://din-lb.solanarpc.dev/KsnzZimk2FZ7c4AHPd3EjGLuXjVnRZ5v3X3mgkq";

        if self.rpc_type.is_some() {
            let prop = self.rpc_type.as_ref().unwrap();
            if prop == "dedicated" {
                rpc_url = "http://66.248.205.6:8899"
            }

            if prop == "info_rpc" {
                rpc_url = "https://api.solanarpc.dev/rpc/solana/mainnet?token=MjI4fE8yeW0zN0s3T251QnY5V1FMcXF4eGRxdVFNbVlaeUYxYWZXRGJLN0U";
            }
        }

        // println!("testing:");

        // println!("RPC URL: {:#?}", rpc_url);
        Ok(RpcClient::new_with_commitment(
            // cluster,
            // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
            rpc_url.to_string(),
            // "http://66.248.205.6:8899",
            // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
            solana_sdk::commitment_config::CommitmentConfig::finalized(),
        ))
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

pub type RpcPool = managed::Pool<RpcPoolManager>;

pub async fn get_pub_sub_client() -> PubsubClient {
    let pubsub_client = PubsubClient::new(wss_url).await.unwrap();
    return pubsub_client;
}
