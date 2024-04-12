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

    println!("token_accounts_mint_a: {:#?}", token_accounts_mint_a);
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
