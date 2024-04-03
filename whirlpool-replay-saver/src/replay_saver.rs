pub mod instruction_callback {
    use anchor_lang::AccountDeserialize;
    use mpl_token_metadata::{
        pda::find_metadata_account,
        state::{Metadata, TokenMetadataAccount},
    };
    use rust_decimal::prelude::*;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{account_info::AccountInfo, pubkey::Pubkey};
    use std::collections::HashMap;
    use whirlpool_base::state::Whirlpool;
    use whirlpool_replayer::{
        schema::{self, DecodedWhirlpoolInstruction, ParsedInstruction, Transaction},
        ReplayInstructionResult, Slot,
    };

    use chrono::prelude::*;

    use solana_client::rpc_client::RpcClient;

    pub fn parse_instruction(
        _slot: &Slot,
        transaction: &Transaction,
        name: &String,
        instruction: &DecodedWhirlpoolInstruction,
        accounts: &HashMap<String, Vec<u8>>,
        result: &ReplayInstructionResult,
        parsed_instructions: &mut Vec<ParsedInstruction>,
    ) {
        // println!("parse_instruction_new");
        // println!("  replayed instruction: {}", name);

        match instruction {
            schema::DecodedWhirlpoolInstruction::Swap(params) => {
                // accounts provides "post" state
                // note: accounts contains all whirlpool accounts at the end of the instruction
                let post_data = accounts.get(&params.key_whirlpool).unwrap();

                let post_whirlpool = Whirlpool::try_deserialize(&mut post_data.as_slice()).unwrap();

                // we can get "pre" state from result
                // note: snapshot only contains whirlpool accounts mentioned in the instruction
                let pre_data = result
                    .snapshot
                    .pre_snapshot
                    .get(&params.key_whirlpool)
                    .unwrap();
                let pre_whirlpool = Whirlpool::try_deserialize(&mut pre_data.as_slice()).unwrap();

                let naive = DateTime::from_timestamp(_slot.block_time, 0);

                let testing = naive.unwrap().format("%Y-%m-%d %H:%M:%S");

                // println!("tick spacing {}", pre_whirlpool.tick_spacing.clone());

                let parsed_instruction: ParsedInstruction = ParsedInstruction {
                    signature: transaction.signature.clone(),
                    instruction_type: name.clone(),
                    pool_address: params.key_whirlpool.clone(),
                    swap_a_ab: if params.data_a_to_b { true } else { false },
                    fee_rate: pre_whirlpool.fee_rate.to_i16().unwrap(), //.to_i8().unwrap(),
                    amount_in: params.transfer_amount_0.to_i64().unwrap(),
                    amount_out: params.transfer_amount_1.to_i64().unwrap(),
                    tick_spacing: pre_whirlpool.tick_spacing.to_i16().unwrap_or(0),
                    sqrt_price_pre: pre_whirlpool.sqrt_price.to_i64().unwrap_or(0),
                    sqrt_price_post: post_whirlpool.sqrt_price.to_i64().unwrap_or(0),
                    token_a: params.key_vault_a.clone(),
                    token_b: params.key_vault_b.clone(),
                    price: "todo".to_string(),
                    date_time: testing.to_string(),
                    // price: get_price_from_tick_price(pre_whirlpool.sqrt_price, 9, 6),
                };

                parsed_instructions.push(parsed_instruction);
            }
            _ => {}
        }
    }

    // pub const parse_instruction: Option<InstructionCallback> = Some(
    //     |_slot, transaction, name, instruction, accounts, result, testArray| {
    //         println!("  replayed instruction: {}", name);

    //         match instruction {
    //             schema::DecodedWhirlpoolInstruction::Swap(params) => {
    //                 // accounts provides "post" state
    //                 // note: accounts contains all whirlpool accounts at the end of the instruction
    //                 let post_data = accounts.get(&params.key_whirlpool).unwrap();
    //                 let post_whirlpool =
    //                     Whirlpool::try_deserialize(&mut post_data.as_slice()).unwrap();

    //                 // we can get "pre" state from result
    //                 // note: snapshot only contains whirlpool accounts mentioned in the instruction
    //                 let pre_data = result
    //                     .snapshot
    //                     .pre_snapshot
    //                     .get(&params.key_whirlpool)
    //                     .unwrap();
    //                 let pre_whirlpool =
    //                     Whirlpool::try_deserialize(&mut pre_data.as_slice()).unwrap();

    //                 let mut objectValues = HashMap::new();

    //                 // objectValues["tx_signature"] = transaction.signature;
    //                 objectValues
    //                     .insert(String::from("tx_signature"), transaction.signature.clone());
    //                 objectValues.insert(String::from("pool"), params.key_whirlpool.clone());
    //                 objectValues.insert(
    //                     String::from("direction"),
    //                     if params.data_a_to_b {
    //                         "A to B".to_string()
    //                     } else {
    //                         "B to A".to_string()
    //                     },
    //                 );
    //                 objectValues.insert(
    //                     String::from("in/out"),
    //                     format!(
    //                         "in={} out={}",
    //                         params.transfer_amount_0, params.transfer_amount_1
    //                     ),
    //                 );
    //                 // objectValues.insert(String::from("sqrt_price"), format!("pre={} post={}", pre_whirlpool.sqrt_price, post_whirlpool.sqrt_price));
    //                 objectValues.insert(
    //                     String::from("fee_rate_pre"),
    //                     pre_whirlpool.fee_rate.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("fee_rate_post"),
    //                     post_whirlpool.fee_rate.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("tick_spacing_pre"),
    //                     pre_whirlpool.tick_spacing.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("tick_spacing_post"),
    //                     post_whirlpool.tick_spacing.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("sqrt_price_pre"),
    //                     pre_whirlpool.sqrt_price.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("sqrt_price_post"),
    //                     post_whirlpool.sqrt_price.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("transfer_amount_in"),
    //                     params.transfer_amount_0.to_string(),
    //                 );
    //                 objectValues.insert(
    //                     String::from("transfer_amount_out"),
    //                     params.transfer_amount_1.to_string(),
    //                 );
    //                 objectValues.insert(String::from("key_vault_a"), params.key_vault_a.clone());
    //                 objectValues.insert(String::from("key_vault_b"), params.key_vault_b.clone());

    //                 // let pre_whirlpool = Whirlpool::try_deserialize(&mut pre_data.as_slice()).unwrap();
    //                 println!("==============================================");
    //                 println!("    tx signature: {}", transaction.signature);
    //                 println!(
    //                     "    pool: {} (ts={}, fee_rate={})",
    //                     params.key_whirlpool, pre_whirlpool.tick_spacing, pre_whirlpool.fee_rate
    //                 );
    //                 println!(
    //                     "      direction: {}",
    //                     if params.data_a_to_b {
    //                         "A to B"
    //                     } else {
    //                         "B to A"
    //                     }
    //                 );
    //                 println!(
    //                     "      in/out: in={} out={}",
    //                     params.transfer_amount_0, params.transfer_amount_1
    //                 );
    //                 println!(
    //                     "      sqrt_price: pre={} post={}",
    //                     pre_whirlpool.sqrt_price, post_whirlpool.sqrt_price
    //                 );
    //                 println!("{:#?}", params);
    //                 println!("==============================================");
    //                 println!("{:#?}", transaction);
    //                 println!("==============================================");
    //                 println!("objectValues {:#?}", objectValues);
    //                 println!("===================from tester======================");
    //                 println!("testArray {:#?} 33", testArray);
    //             }
    //             _ => {}
    //         }
    //     },
    // );
    // fn get_price_from_tick_price() -> Result<(std::string::String), Error> {
    pub fn get_price_from_tick_price(
        sqrt_price_x64: i64,
        decimals_a: u8,
        decimals_b: u8,
    ) -> String {
        if decimals_b > decimals_a {
            return "0".to_string();
        }
        let sqrt_price_x64_decimal = Decimal::from_str(&sqrt_price_x64.to_string()).unwrap();

        let price = sqrt_price_x64_decimal
            .checked_div(Decimal::TWO.powu(64))
            .unwrap()
            .powu(2)
            .checked_mul(Decimal::TEN.powi((decimals_a - decimals_b) as i64))
            .unwrap();

        price.to_string()
    }

    pub fn get_decimals_from_token(connection: &RpcClient, token: &str) -> u8 {
        let pub_key = Pubkey::from_str(token).unwrap();

        let spl_token_program_id: Pubkey = solana_program::pubkey!(pub_key);

        let doalradsfsd = connection.get_token_supply(&pub_key);

        if doalradsfsd.is_err() {
            let token_account = connection.get_token_account(&spl_token_program_id);

            if token_account.is_err() {
                println!("error getting token account for token: {:?}", token);
                return 0;
            }

            let token_account_data = token_account.unwrap();

            match token_account_data {
                None => {
                    println!("No decimals found");
                    return 0;
                }
                Some(name) => {
                    // println!("hello {name}");
                    return name.token_amount.decimals;
                }
            };
        }

        println!("balance accounts from lib: {:#?}", doalradsfsd);

        return doalradsfsd.unwrap().decimals;
    }

    pub fn _get_token_metadata(connection: &RpcClient, token: &str) -> Metadata {
        let pub_key = Pubkey::from_str(token).unwrap();

        let spl_token_program_id: Pubkey = solana_program::pubkey!(pub_key);

        let (meta_data_account, _other) = find_metadata_account(&spl_token_program_id);

        let testing_account = connection.get_account(&meta_data_account);

        let mut account_info = testing_account.unwrap();

        let adf = AccountInfo::new(
            &pub_key,
            false,
            false,
            &mut account_info.lamports,
            &mut account_info.data,
            &mut account_info.owner,
            true,
            account_info.rent_epoch,
        );

        let metadata: Result<Metadata, ProgramError> = Metadata::from_account_info(&adf);

        let metdata_data = metadata.unwrap();

        return metdata_data;
    }

    // return Ok("1000".to_string());
    // }
}
