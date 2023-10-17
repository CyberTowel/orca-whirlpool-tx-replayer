use solana_sdk::pubkey::Pubkey;

use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

use poc_framework::{LocalEnvironment, LocalEnvironmentBuilder};

use crate::errors::ErrorCode;
use crate::{decoded_instructions::DecodedWhirlpoolInstruction, types::AccountMap};
use crate::util_replay;

use crate::programs;
use crate::replay_instructions;
use crate::util_bank;

pub struct WritableAccountSnapshot {
  pub pre_snapshot: AccountMap,
  pub post_snapshot: AccountMap,
}

pub struct ReplayInstructionResult {
  pub transaction_status: EncodedConfirmedTransactionWithStatusMeta,
  pub snapshot: WritableAccountSnapshot,
}

pub struct ReplayInstructionParams<'info, T> {
  pub replayer: &'info mut util_bank::ReplayEnvironment,
  pub decoded_instruction: &'info T,
  pub account_map: &'info AccountMap,
}

const SPL_TOKEN_PROGRAM_ID: Pubkey = solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const ORCA_WHIRLPOOL_PROGRAM_ID: Pubkey = solana_program::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
const METAPLEX_METADATA_PROGRAM_ID: Pubkey = solana_program::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub fn replay_whirlpool_instruction(
  replayer: &mut util_bank::ReplayEnvironment,
  instruction: DecodedWhirlpoolInstruction,
  account_map: &AccountMap, // readonly
) -> Result<ReplayInstructionResult, ErrorCode> {
  match instruction {
    // major instructions
    DecodedWhirlpoolInstruction::Swap(decoded) => Ok(replay_instructions::swap::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::TwoHopSwap(decoded) => Ok(replay_instructions::two_hop_swap::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::UpdateFeesAndRewards(decoded) => Ok(replay_instructions::update_fees_and_rewards::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::CollectFees(decoded) => Ok(replay_instructions::collect_fees::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::CollectReward(decoded) => Ok(replay_instructions::collect_reward::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::CollectProtocolFees(decoded) => Ok(replay_instructions::collect_protocol_fees::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::IncreaseLiquidity(decoded) => Ok(replay_instructions::increase_liquidity::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::DecreaseLiquidity(decoded) => Ok(replay_instructions::decrease_liquidity::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::OpenPosition(decoded) => Ok(replay_instructions::open_position::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::OpenPositionWithMetadata(decoded) => Ok(replay_instructions::open_position_with_metadata::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::ClosePosition(decoded) => Ok(replay_instructions::close_position::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::OpenBundledPosition(decoded) => Ok(replay_instructions::open_bundled_position::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::CloseBundledPosition(decoded) => Ok(replay_instructions::close_bundled_position::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::InitializeTickArray(decoded) => Ok(replay_instructions::initialize_tick_array::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    // minor instructions
    DecodedWhirlpoolInstruction::InitializePool(decoded) => Ok(replay_instructions::initialize_pool::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::InitializeReward(decoded) => Ok(replay_instructions::initialize_reward::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::SetRewardEmissions(decoded) => Ok(replay_instructions::set_reward_emissions::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::InitializePositionBundle(decoded) => Ok(replay_instructions::initialize_position_bundle::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::InitializePositionBundleWithMetadata(decoded) => Ok(replay_instructions::initialize_position_bundle_with_metadata::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),
    DecodedWhirlpoolInstruction::DeletePositionBundle(decoded) => Ok(replay_instructions::delete_position_bundle::replay(ReplayInstructionParams { replayer, decoded_instruction: &decoded, account_map })),

    // ---------------------------------
    // very rare instructions
    // InitializeConfig
    // InitializeFeeTier
    // SetCollectProtocolFeesAuthority
    // SetDefaultFeeRate
    // SetDefaultProtocolFeeRate
    // SetFeeAuthority
    // SetFeeRate
    // SetProtocolFeeRate
    // SetRewardAuthority
    // SetRewardAuthorityBySuperAuthority
    // SetRewardEmissionsSuperAuthority
    // AdminIncreaseLiquidity
    _ => {
      
      Err(ErrorCode::UnknownWhirlpoolInstruction("not implemented yet".to_string()))
    }
  }
}
