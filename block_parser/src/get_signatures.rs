use std::str::FromStr;

use deadpool::managed::Pool;
use solana_client::{
    rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::rpc_pool_manager::RpcPoolManager;

pub async fn get_paginated_singatures(
    address: &str,
    pool: Pool<RpcPoolManager>,
    before_signature_param: Option<String>,
    sample_rate: Option<usize>,
) -> (Vec<String>, Option<String>) {
    let process_interval = 20;
    let remove_error = false;
    // let sample_rate = 30;

    let mut before_signature: Option<Signature> = None;

    if before_signature_param.is_some() {
        before_signature = Some(Signature::from_str(&before_signature_param.unwrap()).unwrap());
    }

    // Signature::from_str(before_signature).unwrap();

    //  Some(Signature::from_str(
    //     "4KfkEVp2QMCM4vEsJgE3fWKuXZmpsv1ema7uBkcHjU4uoM9tVVwuSdPmynx5zJC4mPfirm9mJJCRGT1NRQE2euPA",
    // )
    // .unwrap());

    let pool_pubkey = Pubkey::from_str(address).unwrap();

    let has_more = true;

    let mut all_signatures: Vec<String> = Vec::new();

    while has_more == true {
        let signature_pagination_config: GetConfirmedSignaturesForAddress2Config =
            GetConfirmedSignaturesForAddress2Config {
                commitment: None,
                before: before_signature,
                limit: Some(1000),
                until: None,
            };

        let rpc_connection = pool.clone().get().await.unwrap();

        let signatures_to_process = rpc_connection
            .get_signatures_for_address_with_config(&pool_pubkey, signature_pagination_config)
            .await
            .unwrap();

        if signatures_to_process.last().is_some() {
            let last_signature = &signatures_to_process.last().unwrap().signature;
            before_signature = Option::Some(Signature::from_str(&last_signature).unwrap());
        }

        if signatures_to_process.len() == 0 {
            break;
        }

        let testing: Vec<RpcConfirmedTransactionStatusWithSignature> = signatures_to_process
            .into_iter()
            .filter(|cts: &RpcConfirmedTransactionStatusWithSignature| {
                remove_error || cts.err.is_none()
            })
            .collect();

        let step_by_value = if sample_rate.is_some() {
            let interval = (testing.len() as f64 / sample_rate.unwrap() as f64) as f64;

            let mut selit = interval.floor() as usize;

            if selit < 1 {
                selit = 1;
            }
            selit
        } else {
            1
        };

        let dolar: Vec<String> = testing
            .into_iter()
            .step_by(step_by_value)
            .map(|item| item.signature)
            .collect();

        all_signatures.extend(dolar.clone());

        // let duration = start.elapsed();

        // dbgtest!([dolar.len(), all_signatures.len()]);

        if all_signatures.len() > process_interval + 1 {
            // has_more = false;
            break;
        }

        // let last_signature =
        //     Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());
    }

    // let items: Vec<String> = all_signatures.iter().take(101).collect();

    if process_interval > all_signatures.len() {
        return (all_signatures, None);
    }

    let (item_to_process, b) = all_signatures.split_at(process_interval);

    let next_item = Option::from(b[0].to_string());

    return (item_to_process.to_vec(), next_item);
}
