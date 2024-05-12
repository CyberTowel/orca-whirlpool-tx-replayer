// use crate::state::LiquidityStateLayoutV4;
use borsh::{BorshDeserialize, BorshSerialize};
use deadpool::managed::Pool;
use moka::future::Cache;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
// use solana_sdk::client::SyncClient;
use std::{str::FromStr, time::Duration};

use crate::token_parser::PoolMeta;

/// See https://github.com/raydium-io/raydium-sdk/blob/master/src/liquidity/layout.ts
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LiquidityStateLayoutV4 {
    pub status: u64,
    pub nonce: u64,
    pub max_order: u64,
    pub depth: u64,
    /// minimal decimal step amid orders in relation to decimals of relevant mint
    pub base_decimal: u64,
    pub quote_decimal: u64,
    pub state: u64,
    pub reset_flag: u64,
    /// min size of trade in quote
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimal_value: u64,
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub base_need_take_pnl: u64,
    pub quote_need_take_pnl: u64,
    /// accrued not yet withdraw fee of quote
    pub quote_total_pnl: u64,
    /// accrued not yet withdraw fee of base
    pub base_total_pnl: u64,
    pub quote_total_deposited: u128,
    pub base_total_deposited: u128,
    pub swap_base_in_amount: u128,
    pub swap_quote_out_amount: u128,
    // total fee accrued
    pub swap_base2_quote_fee: u64,
    pub swap_quote_in_amount: u128,
    pub swap_base_out_amount: u128,
    // total fee accrued
    pub swap_quote2_base_fee: u64,
    // amm vault
    /// base spl token account
    pub base_vault: Pubkey,
    /// quite spl token account
    pub quote_vault: Pubkey,
    // mint
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    // market
    /// orders on market done by this pool
    pub open_orders: Pubkey,
    /// usually order book, usually serum
    pub market_id: Pubkey,
    pub market_program_id: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_vault: Pubkey,
    pub owner: Pubkey,
    pub pnl_owner: Pubkey,
}

pub struct PoolMetaBase {
    quote_mint: Pubkey,
    base_mint: Pubkey,
    quote_vault: Pubkey,
    base_vault: Pubkey,
    lp_mint: Pubkey,
}

// pub fn pool_meta_token()

pub async fn get_pool_meta(pool_id: &String, rpc_connection: &RpcClient) -> Option<PoolMeta> {
    // println!("start request: {:?}", pool_id);

    let pubkey = Pubkey::from_str(pool_id).unwrap();

    let state_req = rpc_connection.get_account_data(&pubkey).await;

    if (state_req.is_err()) {
        return None;
    }

    let state = state_req.unwrap();

    let state_parsed = LiquidityStateLayoutV4::deserialize(&mut &state[..]).unwrap();

    let base_decimal = state_parsed.base_decimal;
    let base_lot_size = state_parsed.base_lot_size;
    let base_need_take_pnl = state_parsed.base_need_take_pnl;
    let base_total_pnl = state_parsed.base_total_pnl;
    let base_total_deposited = state_parsed.base_total_deposited;
    let base_vault = state_parsed.base_vault;
    let base_mint = state_parsed.base_mint;

    let quote_decimal = state_parsed.quote_decimal;
    let quote_lot_size = state_parsed.quote_lot_size;
    let quote_need_take_pnl = state_parsed.quote_need_take_pnl;
    let quote_total_pnl = state_parsed.quote_total_pnl;
    let quote_total_deposited = state_parsed.quote_total_deposited;
    let quote_vault = state_parsed.quote_vault;
    let quote_mint = state_parsed.quote_mint;

    if base_mint.to_string() == "So11111111111111111111111111111111111111112"
        && quote_mint.to_string() != "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    {
        let info = PoolMeta {
            base_decimal: quote_decimal,
            base_lot_size: quote_lot_size,
            base_need_take_pnl: quote_need_take_pnl,
            base_total_pnl: quote_total_pnl,
            base_total_deposited: quote_total_deposited,
            base_vault: quote_vault,
            base_mint: quote_mint,
            quote_decimal: base_decimal,
            quote_lot_size: base_lot_size,
            quote_need_take_pnl: base_need_take_pnl,
            quote_total_pnl: base_total_pnl,
            quote_total_deposited: base_total_deposited,
            quote_vault: base_vault,
            quote_mint: base_mint,
        };

        return Some(info);
    }

    let meta = PoolMeta {
        base_decimal,
        base_lot_size,
        base_need_take_pnl,
        base_total_pnl,
        base_total_deposited,
        base_vault,
        base_mint,
        quote_decimal,
        quote_lot_size,
        quote_need_take_pnl,
        quote_total_pnl,
        quote_total_deposited,
        quote_vault,
        quote_mint,
    };

    return Some(meta);

    // return state;
}

// pub fn get_pool_state(pool_id: String) -> LiquidityStateLayoutV4 {
//     // LiquidityStateLayoutV4::try_from_slice(data).unwrap()
//     let state = state(&pool_id);
//     return state;
// }

async fn state(pool_id: &String, rpc_connection: &RpcClient) -> Option<LiquidityStateLayoutV4> {
    let ref pool = pool_id.parse().unwrap();
    // let solana = RpcClient::new_with_timeout(
    //     "https://api.solanarpc.dev/rpc/solana/mainnet?token=MjI4fE8yeW0zN0s3T251QnY5V1FMcXF4eGRxdVFNbVlaeUYxYWZXRGJLN0U".to_string(),
    //     Duration::from_secs(120),
    // );

    // let client = RpcClient::new_with_commitment(
    //     // cluster,
    //     // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
    //     "https://api.solanarpc.dev/rpc/solana/mainnet?token=MjI4fE8yeW0zN0s3T251QnY5V1FMcXF4eGRxdVFNbVlaeUYxYWZXRGJLN0U".to_string(),
    //     // "http://66.248.205.6:8899",
    //     // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
    //     solana_sdk::commitment_config::CommitmentConfig::finalized(),
    // );

    println!("get info");

    let pool_req = rpc_connection.get_account_data(pool).await;

    println!("testing after");

    if (pool_req.is_err()) {
        println!("Error: {:?}", pool_req.err().unwrap());
        return None;
    }

    println!("sucesss!! ");

    let pool: Vec<u8> = pool_req.unwrap();
    let data = LiquidityStateLayoutV4::deserialize(&mut &pool[..]).unwrap();

    // dbg!("{:?}", data);

    return Some(data);
}
