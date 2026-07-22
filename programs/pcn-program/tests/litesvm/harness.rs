#![allow(dead_code)]

use {
    anchor_lang::{
        solana_program::{
            bpf_loader_upgradeable::{self, UpgradeableLoaderState},
            instruction::Instruction,
            system_instruction,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::{self, instruction as token_instruction},
        TokenAccount,
    },
    litesvm::{types::TransactionResult, LiteSVM},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    std::path::PathBuf,
};

pub struct PcnTestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub admin: Keypair,
    pub oracle: Keypair,
    pub mint: Keypair,
    pub config: anchor_lang::prelude::Pubkey,
    pub mint_authority: anchor_lang::prelude::Pubkey,
    pub sol_reserve: anchor_lang::prelude::Pubkey,
    pub token_reserve_vault: anchor_lang::prelude::Pubkey,
    pub curve: pcn_program::CurveParams,
}

pub struct EpochFixture {
    pub epoch_id: u64,
    pub epoch: anchor_lang::prelude::Pubkey,
    pub epoch_token_vault: anchor_lang::prelude::Pubkey,
    pub user_one: Keypair,
    pub user_two: Keypair,
    pub user_one_token: Keypair,
    pub user_two_token: Keypair,
}

pub fn setup_pcn_litesvm() -> Option<PcnTestContext> {
    let mut ctx = setup_uninitialized_pcn_litesvm();
    initialize_config_result(&mut ctx, program_data_address()).unwrap();
    Some(ctx)
}

pub fn setup_uninitialized_pcn_litesvm() -> PcnTestContext {
    let so_path = program_so_path().unwrap_or_else(|| {
        panic!(
            "missing target/deploy/pcn_program.so; run `NO_DNA=1 anchor build` before `cargo test`"
        )
    });
    let program_id = pcn_program::id();
    let payer = Keypair::new();
    let admin = Keypair::new();
    let oracle = Keypair::new();
    let mint = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = std::fs::read(so_path).unwrap();
    svm.add_program(program_id, &bytes).unwrap();
    set_program_upgrade_authority(&mut svm, payer.pubkey());
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let config = pda(&[pcn_program::CONFIG_SEED]);
    let mint_authority = pda(&[pcn_program::MINT_AUTHORITY_SEED]);
    let sol_reserve = pda(&[pcn_program::SOL_RESERVE_SEED]);
    let token_reserve_vault = pda(&[pcn_program::TOKEN_RESERVE_SEED]);
    let curve = test_curve();

    PcnTestContext {
        svm,
        payer,
        admin,
        oracle,
        mint,
        config,
        mint_authority,
        sol_reserve,
        token_reserve_vault,
        curve,
    }
}

fn initialize_config_result(
    ctx: &mut PcnTestContext,
    program_data: anchor_lang::prelude::Pubkey,
) -> TransactionResult {
    let init_ix = Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::InitializeConfig {
            args: pcn_program::InitializeConfigArgs {
                admin: ctx.admin.pubkey(),
                oracle: ctx.oracle.pubkey(),
                claim_window_slots: 5,
                curve: ctx.curve,
            },
        }
        .data(),
        pcn_program::accounts::InitializeConfig {
            payer: ctx.payer.pubkey(),
            program: pcn_program::id(),
            program_data,
            config: ctx.config,
            reward_mint: ctx.mint.pubkey(),
            mint_authority: ctx.mint_authority,
            sol_reserve: ctx.sol_reserve,
            token_reserve_vault: ctx.token_reserve_vault,
            system_program: anchor_lang::system_program::ID,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    );
    send_result(
        &mut ctx.svm,
        &ctx.payer,
        vec![init_ix],
        &[&ctx.payer, &ctx.mint],
    )
}

fn set_program_upgrade_authority(
    svm: &mut LiteSVM,
    upgrade_authority: anchor_lang::prelude::Pubkey,
) {
    let address = program_data_address();
    let mut account = svm.get_account(&address).unwrap();
    let metadata_len = UpgradeableLoaderState::size_of_programdata_metadata();
    bincode::serialize_into(
        &mut account.data[..metadata_len],
        &UpgradeableLoaderState::ProgramData {
            slot: 0,
            upgrade_authority_address: Some(upgrade_authority),
        },
    )
    .unwrap();
    svm.set_account(address, account).unwrap();
}

fn program_data_address() -> anchor_lang::prelude::Pubkey {
    anchor_lang::prelude::Pubkey::find_program_address(
        &[pcn_program::id().as_ref()],
        &bpf_loader_upgradeable::ID,
    )
    .0
}

pub fn initialize_config_with_unrelated_program_data(
    ctx: &mut PcnTestContext,
) -> TransactionResult {
    let unrelated_program_data = anchor_lang::prelude::Pubkey::new_unique();
    let canonical_program_data = ctx.svm.get_account(&program_data_address()).unwrap();
    ctx.svm
        .set_account(unrelated_program_data, canonical_program_data)
        .unwrap();

    initialize_config_result(ctx, unrelated_program_data)
}

pub fn create_epoch_fixture(ctx: &mut PcnTestContext, epoch_id: u64) -> EpochFixture {
    let epoch_id_bytes = epoch_id.to_le_bytes();
    let epoch = pda(&[pcn_program::EPOCH_SEED, epoch_id_bytes.as_ref()]);
    let epoch_token_vault = pda(&[pcn_program::EPOCH_VAULT_SEED, epoch_id_bytes.as_ref()]);
    let user_one = Keypair::new();
    let user_two = Keypair::new();
    let user_one_token = Keypair::new();
    let user_two_token = Keypair::new();

    create_token_account(
        &mut ctx.svm,
        &ctx.payer,
        &user_one_token,
        &user_one.pubkey(),
        &ctx.mint.pubkey(),
    );
    create_token_account(
        &mut ctx.svm,
        &ctx.payer,
        &user_two_token,
        &user_two.pubkey(),
        &ctx.mint.pubkey(),
    );

    EpochFixture {
        epoch_id,
        epoch,
        epoch_token_vault,
        user_one,
        user_two,
        user_one_token,
        user_two_token,
    }
}

pub fn open_epoch(ctx: &mut PcnTestContext, epoch: &EpochFixture, support_budget_lamports: u64) {
    let ix = open_epoch_ix(ctx, epoch, ctx.oracle.pubkey(), support_budget_lamports);
    send(
        &mut ctx.svm,
        &ctx.payer,
        vec![ix],
        &[&ctx.payer, &ctx.oracle],
    );
}

pub fn open_epoch_result(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    oracle: &Keypair,
    support_budget_lamports: u64,
) -> TransactionResult {
    let ix = open_epoch_ix(ctx, epoch, oracle.pubkey(), support_budget_lamports);
    send_result(&mut ctx.svm, &ctx.payer, vec![ix], &[&ctx.payer, oracle])
}

pub fn finalize_epoch(ctx: &mut PcnTestContext, epoch: &EpochFixture, total_reward_weight: u64) {
    let ix = finalize_epoch_ix(ctx, epoch, ctx.oracle.pubkey(), total_reward_weight);
    send(
        &mut ctx.svm,
        &ctx.payer,
        vec![ix],
        &[&ctx.payer, &ctx.oracle],
    );
}

pub fn finalize_epoch_result(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    oracle: &Keypair,
    total_reward_weight: u64,
) -> TransactionResult {
    let ix = finalize_epoch_ix(ctx, epoch, oracle.pubkey(), total_reward_weight);
    send_result(&mut ctx.svm, &ctx.payer, vec![ix], &[&ctx.payer, oracle])
}

pub fn create_claim(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    user: anchor_lang::prelude::Pubkey,
    bandwidth_units: u64,
    quality_factor_ppm: u64,
) -> anchor_lang::prelude::Pubkey {
    let claim = claim_pda(epoch.epoch_id, &user);
    let ix = create_claim_ix(ctx, epoch, claim, user, bandwidth_units, quality_factor_ppm);
    send(
        &mut ctx.svm,
        &ctx.payer,
        vec![ix],
        &[&ctx.payer, &ctx.oracle],
    );
    claim
}

pub fn create_claim_result(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    user: anchor_lang::prelude::Pubkey,
    bandwidth_units: u64,
    quality_factor_ppm: u64,
) -> TransactionResult {
    let claim = claim_pda(epoch.epoch_id, &user);
    let ix = create_claim_ix(ctx, epoch, claim, user, bandwidth_units, quality_factor_ppm);
    send_result(
        &mut ctx.svm,
        &ctx.payer,
        vec![ix],
        &[&ctx.payer, &ctx.oracle],
    )
}

pub fn claim_reward(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    claim: anchor_lang::prelude::Pubkey,
    user: &Keypair,
    user_token_account: anchor_lang::prelude::Pubkey,
) {
    let ix = claim_reward_ix(ctx, epoch, claim, user.pubkey(), user_token_account);
    send(&mut ctx.svm, &ctx.payer, vec![ix], &[&ctx.payer, user]);
}

pub fn claim_reward_result(
    ctx: &mut PcnTestContext,
    epoch: &EpochFixture,
    claim: anchor_lang::prelude::Pubkey,
    user: &Keypair,
    user_token_account: anchor_lang::prelude::Pubkey,
) -> TransactionResult {
    let ix = claim_reward_ix(ctx, epoch, claim, user.pubkey(), user_token_account);
    send_result(&mut ctx.svm, &ctx.payer, vec![ix], &[&ctx.payer, user])
}

pub fn sweep_epoch(ctx: &mut PcnTestContext, epoch: &EpochFixture) {
    let ix = sweep_epoch_ix(ctx, epoch, ctx.oracle.pubkey());
    send(
        &mut ctx.svm,
        &ctx.payer,
        vec![ix],
        &[&ctx.payer, &ctx.oracle],
    );
}

pub fn update_config_result(
    ctx: &mut PcnTestContext,
    admin: &Keypair,
    args: pcn_program::UpdateConfigArgs,
) -> TransactionResult {
    let ix = Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::UpdateConfig { args }.data(),
        pcn_program::accounts::UpdateConfig {
            admin: admin.pubkey(),
            config: ctx.config,
        }
        .to_account_metas(None),
    );
    send_result(&mut ctx.svm, &ctx.payer, vec![ix], &[&ctx.payer, admin])
}

pub fn get_anchor_account<T: AccountDeserialize>(
    svm: &LiteSVM,
    address: &anchor_lang::prelude::Pubkey,
) -> T {
    let account = svm.get_account(address).unwrap();
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).unwrap()
}

pub fn token_amount(svm: &LiteSVM, address: &anchor_lang::prelude::Pubkey) -> u64 {
    let account = svm.get_account(address).unwrap();
    let mut data: &[u8] = &account.data;
    TokenAccount::try_deserialize_unchecked(&mut data)
        .unwrap()
        .amount
}

pub fn account_lamports(svm: &LiteSVM, address: &anchor_lang::prelude::Pubkey) -> u64 {
    svm.get_balance(address).unwrap()
}

pub fn claim_pda(
    epoch_id: u64,
    user: &anchor_lang::prelude::Pubkey,
) -> anchor_lang::prelude::Pubkey {
    pda(&[
        pcn_program::CLAIM_SEED,
        epoch_id.to_le_bytes().as_ref(),
        user.as_ref(),
    ])
}

pub fn send(svm: &mut LiteSVM, payer: &Keypair, ixs: Vec<Instruction>, signers: &[&Keypair]) {
    send_result(svm, payer, ixs, signers).unwrap();
}

fn send_result(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: Vec<Instruction>,
    signers: &[&Keypair],
) -> TransactionResult {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn open_epoch_ix(
    ctx: &PcnTestContext,
    epoch: &EpochFixture,
    oracle: anchor_lang::prelude::Pubkey,
    support_budget_lamports: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::OpenEpoch {
            args: pcn_program::OpenEpochArgs {
                epoch_id: epoch.epoch_id,
                start_slot: 0,
                end_slot: 1,
                support_budget_lamports,
            },
        }
        .data(),
        pcn_program::accounts::OpenEpoch {
            oracle,
            funder: ctx.payer.pubkey(),
            config: ctx.config,
            epoch: epoch.epoch,
            epoch_token_vault: epoch.epoch_token_vault,
            mint_authority: ctx.mint_authority,
            reward_mint: ctx.mint.pubkey(),
            system_program: anchor_lang::system_program::ID,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

fn finalize_epoch_ix(
    ctx: &PcnTestContext,
    epoch: &EpochFixture,
    oracle: anchor_lang::prelude::Pubkey,
    total_reward_weight: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::FinalizeEpoch {
            args: pcn_program::FinalizeEpochArgs {
                total_reward_weight,
            },
        }
        .data(),
        pcn_program::accounts::FinalizeEpoch {
            oracle,
            refund_target: ctx.payer.pubkey(),
            config: ctx.config,
            epoch: epoch.epoch,
            epoch_token_vault: epoch.epoch_token_vault,
            reward_mint: ctx.mint.pubkey(),
            mint_authority: ctx.mint_authority,
            sol_reserve: ctx.sol_reserve,
            token_program: spl_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn create_claim_ix(
    ctx: &PcnTestContext,
    epoch: &EpochFixture,
    claim: anchor_lang::prelude::Pubkey,
    user: anchor_lang::prelude::Pubkey,
    bandwidth_units: u64,
    quality_factor_ppm: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::CreateClaim {
            args: pcn_program::CreateClaimArgs {
                epoch_id: epoch.epoch_id,
                user,
                bandwidth_units,
                quality_factor_ppm,
            },
        }
        .data(),
        pcn_program::accounts::CreateClaim {
            oracle: ctx.oracle.pubkey(),
            payer: ctx.payer.pubkey(),
            config: ctx.config,
            epoch: epoch.epoch,
            claim,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn claim_reward_ix(
    ctx: &PcnTestContext,
    epoch: &EpochFixture,
    claim: anchor_lang::prelude::Pubkey,
    user: anchor_lang::prelude::Pubkey,
    user_token_account: anchor_lang::prelude::Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::ClaimReward {
            args: pcn_program::ClaimRewardArgs {
                epoch_id: epoch.epoch_id,
            },
        }
        .data(),
        pcn_program::accounts::ClaimReward {
            user,
            config: ctx.config,
            epoch: epoch.epoch,
            claim,
            epoch_token_vault: epoch.epoch_token_vault,
            user_token_account,
            mint_authority: ctx.mint_authority,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

fn sweep_epoch_ix(
    ctx: &PcnTestContext,
    epoch: &EpochFixture,
    oracle: anchor_lang::prelude::Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        pcn_program::id(),
        &pcn_program::instruction::SweepEpoch {
            args: pcn_program::SweepEpochArgs {
                epoch_id: epoch.epoch_id,
            },
        }
        .data(),
        pcn_program::accounts::SweepEpoch {
            oracle,
            config: ctx.config,
            epoch: epoch.epoch,
            epoch_token_vault: epoch.epoch_token_vault,
            token_reserve_vault: ctx.token_reserve_vault,
            mint_authority: ctx.mint_authority,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

fn create_token_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    token_account: &Keypair,
    owner: &anchor_lang::prelude::Pubkey,
    mint: &anchor_lang::prelude::Pubkey,
) {
    let rent = svm.minimum_balance_for_rent_exemption(TokenAccount::LEN);
    send(
        svm,
        payer,
        vec![
            system_instruction::create_account(
                &payer.pubkey(),
                &token_account.pubkey(),
                rent,
                TokenAccount::LEN as u64,
                &spl_token::ID,
            ),
            token_instruction::initialize_account3(
                &spl_token::ID,
                &token_account.pubkey(),
                mint,
                owner,
            )
            .unwrap(),
        ],
        &[payer, token_account],
    );
}

fn pda(seeds: &[&[u8]]) -> anchor_lang::prelude::Pubkey {
    anchor_lang::prelude::Pubkey::find_program_address(seeds, &pcn_program::id()).0
}

fn program_so_path() -> Option<PathBuf> {
    [
        PathBuf::from("target/deploy/pcn_program.so"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/pcn_program.so"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn test_curve() -> pcn_program::CurveParams {
    pcn_program::CurveParams {
        max_epoch_mint: pcn_program::TOKEN_BASE_UNITS,
        emission_multiplier_ppm: pcn_program::EMISSION_MULTIPLIER_PPM_SCALE,
        saturation_units: 100,
        history_minted: pcn_program::TOKEN_BASE_UNITS,
        target_support_lamports_per_token: 10_000_000,
        max_supply: 10 * pcn_program::TOKEN_BASE_UNITS,
    }
}
