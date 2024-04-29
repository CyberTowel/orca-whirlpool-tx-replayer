pub fn get_accounts() {
    let pubkey_ubo = Pubkey::from_str(&transaction_parsed.ubo).unwrap();
    let pubkey_token_b = Pubkey::from_str(&transaction_parsed.token_b_address).unwrap();

    let pubkey_program_id =
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let token_account_filter = TokenAccountsFilter::ProgramId(pubkey_program_id);
    //::Mint(pubkey_token_b);

    let token_accounts_mint_a = rpc_connection
        .get_token_accounts_by_owner(&pubkey_ubo, token_account_filter)
        .unwrap();

    // let token_account_address_a = &token_accounts_mint_a[0].pubkey;

    // let token_account_address_a_pubkey = Pubkey::from_str(token_account_address_a).unwrap();

    // let hashes = rpc_connection
    //     .get_signatures_for_address(&token_account_address_a_pubkey)
    //     .unwrap();

    // let singatures = hashes
    //     .iter()
    //     .map(|x| x.signature.to_string())
    //     .collect::<Vec<String>>();
}

fn token_info() {
    let pubkey = Pubkey::from_str(&pool_coin_token_account).unwrap();

    let mut account_info = rpc_connection.get_account(&pubkey).unwrap();

    let mut data = account_info.data;

    // let mut lamport1 = &mut account_info.clone().lamports;

    let accounttesting = AccountInfo::new(
        &pubkey,
        true,
        true,
        &mut account_info.lamports,
        &mut data,
        &account_info.owner,
        account_info.executable,
        account_info.rent_epoch,
    );

    let dolar = accounttesting.data.as_ref().borrow().to_owned();

    let additional_data = get_token_account_mint(&dolar)
        .map(|key| get_mint_decimals(&accounttesting).ok())
        .map(|decimals| AccountAdditionalData {
            spl_token_decimals: decimals,
        });

    // let additional_data = Some(AccountAdditionalData {
    //     spl_token_decimals: Some(8),
    // });

    // let dadf = additional_data.unwrap();

    let testing_owner = Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap();

    let testing =
        parse_account_data(&pubkey, &accounttesting.owner, &dolar, additional_data).unwrap();
}
