#[path = "harness.rs"]
mod harness;

use harness::*;
use solana_signer::Signer;

#[test]
fn happy_path_claims_and_sweeps_dust_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let epoch = create_epoch_fixture(&mut ctx, 1);
    let reserve_before = account_lamports(&ctx.svm, &ctx.sol_reserve);

    open_epoch(&mut ctx, &epoch, 10_000_000);
    ctx.svm.warp_to_slot(2);
    finalize_epoch(&mut ctx, &epoch, 100);

    let finalized: pcn_program::Epoch = get_anchor_account(&ctx.svm, &epoch.epoch);
    assert_eq!(finalized.reward_pool_amount, 632_121);
    assert_eq!(finalized.total_reward_weight, 100);
    assert_eq!(
        account_lamports(&ctx.svm, &ctx.sol_reserve) - reserve_before,
        finalized.consumed_support_lamports
    );

    let claim_one = create_claim(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(100, 0, 100, 0),
    );
    let claim_two = create_claim(
        &mut ctx,
        &epoch,
        epoch.user_two.pubkey(),
        performance(0, 100, 0, 100),
    );

    claim_reward(
        &mut ctx,
        &epoch,
        claim_one,
        &epoch.user_one,
        epoch.user_one_token.pubkey(),
    );
    claim_reward(
        &mut ctx,
        &epoch,
        claim_two,
        &epoch.user_two,
        epoch.user_two_token.pubkey(),
    );

    assert_eq!(
        token_amount(&ctx.svm, &epoch.user_one_token.pubkey()),
        316_060
    );
    assert_eq!(
        token_amount(&ctx.svm, &epoch.user_two_token.pubkey()),
        316_060
    );
    assert_eq!(token_amount(&ctx.svm, &epoch.epoch_token_vault), 1);

    ctx.svm.warp_to_slot(finalized.claim_deadline_slot + 1);
    sweep_epoch(&mut ctx, &epoch);

    let swept: pcn_program::Epoch = get_anchor_account(&ctx.svm, &epoch.epoch);
    assert_eq!(swept.status, pcn_program::EpochStatus::Swept);
    assert_eq!(token_amount(&ctx.svm, &epoch.epoch_token_vault), 0);
    assert_eq!(token_amount(&ctx.svm, &ctx.token_reserve_vault), 1);
}
