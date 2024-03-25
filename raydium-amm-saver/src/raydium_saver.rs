pub mod raydium {

    use chrono::prelude::*;
    use rust_decimal::prelude::*;
    use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
    use solana_sdk::{pubkey::Pubkey, signature::Signature};
    use solana_transaction_status::{UiTransactionEncoding, UiTransactionTokenBalance};

    pub fn parse_signature(signature: &String, rpc_connection: &RpcClient) {
        // println!(
        //     "======================================= signature: {} ========================================",
        //     signature.signature
        // );

        let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: Some(1),
        };

        let transaction = rpc_connection
            .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

        let testing_blocktime = transaction.as_ref().unwrap().block_time.unwrap();

        let dolar: Option<Vec<UiTransactionTokenBalance>> = transaction
            .unwrap()
            .transaction
            .meta
            .unwrap()
            .post_token_balances
            .into();

        let mut amount_token_a = &dolar.clone().unwrap();

        let amount_token_a_test = &amount_token_a
            .iter()
            .find(|&x| {
                let owner: Option<String> = x.owner.clone().into();
                x.mint == "So11111111111111111111111111111111111111112"
                    && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let amount_token_b_test = &amount_token_a
            .iter()
            .find(|&x| {
                let owner: Option<String> = x.owner.clone().into();
                x.mint == "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J"
                    && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
                // && Some(x.owner.into()) == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let price = get_price(amount_token_a_test, amount_token_b_test);

        println!(
            "price: {}, time: {}, signature: {}, token_a: {}, token_b: {}",
            price,
            DateTime::from_timestamp(testing_blocktime, 0).unwrap(),
            signature,
            amount_token_a_test,
            amount_token_b_test,
        );
    }

    fn get_price(token_a_balance: &String, token_b_balance: &String) -> Decimal {
        let token_a_price = Decimal::from_str(&token_a_balance.to_string()).unwrap();
        let token_b_price = Decimal::from_str(&token_b_balance.to_string()).unwrap();

        let price2 = token_a_price
            .checked_div(token_b_price)
            .unwrap()
            .checked_mul(Decimal::TEN.powi((-6) as i64))
            .unwrap();

        price2
    }
}
