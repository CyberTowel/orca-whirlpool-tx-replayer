mod raydium_saver;
use clap::Parser;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::{Error, NoTls};
// use anchor_lang::solana_program::pubkey::Pubkey;
use chrono::prelude::*;
use raydium_saver::raydium::parse_signature;
use rust_decimal::prelude::*;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, id = "directory")]
    cache_dir: Option<String>,

    #[clap(short, long, id = "filename")]
    save_as: Option<String>,

    #[clap(long, id = "slot")]
    stop_slot: Option<u64>,

    #[clap(long, id = "lieke")]
    lieke: Option<String>,

    #[clap(long, id = "blockHeight")]
    stop_block_height: Option<u64>,

    #[clap(long, id = "blockTime")]
    stop_block_time: Option<i64>,

    #[clap(id = "path|url")]
    storage: String,

    #[clap(id = "yyyymmdd")]
    yyyymmdd: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello, world!");

    println!("Hello, world! test");

    let rpc_connection = RpcClient::new_with_commitment(
        // cluster,
        "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );

    //  March 18, 2024 11:23:37 Central European Standard Time
    let signature =
        "edbGpGBuAaFnjde2KU3YgWE1Vw5Kawz19gXKXtc5bPPE3VMVhaGf2Ba3o6Lc3HyqBZJd6vBKT2dyjhjxBgufBA9"
            .to_string();

    parse_signature(&signature, &rpc_connection);

    let price_ref = "00009260";
    println!("price: {}", price_ref);

    return Ok(());

    let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    let pub_key = Pubkey::from_str(pool_id).unwrap();

    let raydium_pool = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

    let timestamp = "March 18, 2024 23:33:27 +UTC";
    let price_usd = "0.02392";
    let price_sol = "0.00009431";

    let testing = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

    let token_a_sol = "Ffo9MEhfH5tBBkZMi1vWVpZLqmbDKvEWJhW3XyMQz4QY";
    let token_b_idk = "EM9ebwJyrenPmgXQyn9aR5X2tiJssrVPwLSZXxmg2dLy";

    // let token_a_balance: i64 = 4075612770818;
    let hash =
        "3Sg9egbG9qYjCAa18G6XiB2JCvvGvD8UiAsL21CAvGotWhuJdvRjE93NTam6ZnkPeTc8b5c3mtAZHjCBhqQus78o";
    let token_b_balance: i64 = 43038823421;
    let token_a_balance: i64 = 4076108970818;

    // 7MWcyzx4UkcYc1DzZUxTY3V1PaLVDzvZnyzT3LAEx47Xj9ecwaU6VVoLSpNij2K5BMzTyPLcJWtXf8obFYLFoiA
    let price_sol2 = "0.000006530";

    let token_b_balance2: i64 = 167786000506;
    let token_a_balance2: i64 = 1116726644981;

    // let price = get_price(token_a_balance, token_b_balance);
    // let price2 = get_price(token_a_balance2, token_b_balance2);

    // for signature in &doalradsfsd {
    //     println!("signature: {}", signature.signature);
    // }

    let mut has_more = true;
    let mut before_signature: Option<Signature> = None;

    let mut before_signature: Option<Signature> = Some(Signature::from_str(
        "5qcAJQKafzci6oDkjwuCmFn8wxnGYnYr1MpED2xsM5SDbgreM1FU6ypsCokQ3Newkvbxqo8TgyjKJ2UbffjZjVHc",
    )
    .unwrap());

    let max_tries = 2;
    let mut round = 0;

    while has_more == true && round < max_tries {
        round += 1;
        let testing: GetConfirmedSignaturesForAddress2Config =
            GetConfirmedSignaturesForAddress2Config {
                commitment: None,
                before: before_signature,
                limit: Some(1000),
                until: None,
            };

        let doalradsfsd = rpc_connection
            .get_signatures_for_address_with_config(&pub_key, testing)
            .unwrap();

        println!("testing");
        // if (doalradsfsd.len() != 1000) {
        //     has_more = false;
        //     break;
        // }

        let testing_dolar =
            Some(Signature::from_str(&doalradsfsd.last().unwrap().signature).unwrap());

        before_signature = testing_dolar;
        // let test_before_signature = doalradsfsd
        //     .last()
        //     .unwrap()
        //     .clone_from(&doalradsfsd.last().unwrap());

        println!("total items : {}", doalradsfsd.len());
        println!(
            "first item : {:?}, time; {:?}",
            doalradsfsd.first().unwrap().signature,
            DateTime::from_timestamp(doalradsfsd.first().unwrap().block_time.unwrap(), 0)
        );

        println!(
            "first item : {:?}, time; {:?}",
            doalradsfsd.last().unwrap().signature,
            DateTime::from_timestamp(doalradsfsd.last().unwrap().block_time.unwrap(), 0)
        );

        for signature in &doalradsfsd {
            parse_signature(&signature.signature, &rpc_connection);
            // println!(
            //     "======================================= signature: {} ========================================",
            //     signature.signature
            // );

            // let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
            //     encoding: Some(UiTransactionEncoding::JsonParsed),
            //     commitment: None,
            //     max_supported_transaction_version: Some(1),
            // };

            // let transaction = rpc_connection.get_transaction_with_config(
            //     &Signature::from_str(&signature.signature).unwrap(),
            //     rpc_config,
            // );

            // let testing_blocktime = transaction.as_ref().unwrap().block_time.unwrap();

            // // let blocktime = DateTime::from_timestamp(transaction.clone().unwrap().block_time.unwrap(), 0);

            // let dolar: Option<Vec<UiTransactionTokenBalance>> = transaction
            //     .unwrap()
            //     .transaction
            //     .meta
            //     .unwrap()
            //     .post_token_balances
            //     .into();

            // let mut amount_token_a = &dolar.clone().unwrap();

            // let amount_token_a_test = &amount_token_a
            //     .iter()
            //     .find(|&x| {
            //         let owner: Option<String> = x.owner.clone().into();
            //         x.mint == "So11111111111111111111111111111111111111112"
            //             && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
            //     })
            //     .unwrap()
            //     .ui_token_amount
            //     .amount;

            // let amount_token_b = &dolar.clone().unwrap();

            // let mut amount_token_b_test = 0;

            // for test in amount_token_a {
            //     if (test.mint == "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J") {
            //         amount_token_b_test += test.ui_token_amount.amount.parse::<i64>().unwrap()
            //     }

            //     // println!(
            //     //     "change: {:#?} to token: {:?} owned_by: {:?}, amount_token_b_test: {}",
            //     //     test.ui_token_amount.amount, test.mint, test.owner, amount_token_b_test
            //     // )
            // }

            // let mut amount_token_a_test = 0;
            // for test in amount_token_a {
            //     if (test.mint == "So11111111111111111111111111111111111111112") {
            //         amount_token_a_test += test.ui_token_amount.amount.parse::<i64>().unwrap()
            //     }

            //     // println!(
            //     //     "change: {:#?} to token: {:?} owned_by: {:?}, amount_token_b_test: {}",
            //     //     test.ui_token_amount.amount, test.mint, test.owner, amount_token_a_test
            //     // )
            // }

            // let amount_token_b_test = &amount_token_b
            //     .iter()
            //     .find(|&x| {
            //         let owner: Option<String> = x.owner.clone().into();
            //         x.mint == "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J"
            //             && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
            //         // && Some(x.owner.into()) == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
            //     })
            //     .unwrap()
            //     .ui_token_amount
            //     .amount;

            // let _amount_token_b = &dolar
            //     .clone()
            //     .unwrap()
            //     .iter()
            //     .find(|&x| x.mint == "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J")
            //     .unwrap()
            //     .ui_token_amount
            //     .amount;

            // let price = get_price(amount_token_a_test, amount_token_b_test);

            // let price = get_price(amount_token_a_test, amount_token_b_test);

            // let timestamp = DateTime::from_timestamp(
            //     // transaction.unwrap().clone().unwrap().block_time.unwrap(),
            //     0,
            // );

            // println!(
            //     "price: {}, time: {}, signature: {}, token_a: {}, token_b: {}",
            //     price,
            //     DateTime::from_timestamp(testing_blocktime, 0).unwrap(),
            //     signature.signature,
            //     amount_token_a_test,
            //     amount_token_b_test,
            // );

            // if (amount_token_a_test == "0" || amount_token_b_test == "0") {
            //     println!("token not found {:#?}", dolar.clone().unwrap());
            // }
            // println!("amount_token_b: {}", _amount_token_b);

            // for change in dolar.clone().unwrap().iter() {
            //     println!(
            //         "change: {:#?} to token: {:?} owned_by: {:?}",
            //         change.ui_token_amount.amount, change.mint, change.owner
            //     );
            // }

            // println!("dolar: {:#?}", dolar.unwrap());

            // println!("dolar: {:#?}", dolar);
        }
    }

    // println!("doalradsfsd: {:#?}", doalradsfsd);

    // println!("testing: {}", price.to_string());
    // println!("testing: {}", price_sol.to_string());

    // println!("testing price 1: {}", price.to_string());
    // println!("testing price 1: {}", price_sol.to_string());

    // println!("testing price 2: {}", price2.to_string());
    // println!("testing price 2: {}", price_sol2.to_string());

    return Ok(());
}
