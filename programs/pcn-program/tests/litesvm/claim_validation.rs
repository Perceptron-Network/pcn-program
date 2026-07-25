#[path = "harness.rs"]
mod harness;

use harness::*;
use solana_signer::Signer;

#[test]
fn create_claim_rejects_early_invalid_metric_and_duplicate_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let epoch = create_epoch_fixture(&mut ctx, 1);
    open_epoch(&mut ctx, &epoch, 10_000_000);

    assert!(create_claim_result(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(49, 49, 49, 49),
    )
    .is_err());

    ctx.svm.warp_to_slot(2);
    finalize_epoch(&mut ctx, &epoch, 100);
    assert!(create_claim_result(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(pcn_program::PERFORMANCE_PPM_SCALE + 1, 50, 50, 50),
    )
    .is_err());

    let claim = create_claim(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(50, 50, 50, 50),
    );
    assert!(create_claim_result(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(50, 50, 50, 50),
    )
    .is_err());

    let claim_state: pcn_program::Claim = get_anchor_account(&ctx.svm, &claim);
    assert_eq!(claim_state.reward_weight, 50);
    assert_eq!(claim_state.reward_amount, 316_060_279);
}

#[test]
fn claim_reward_rejects_wrong_user_and_duplicate_redemption_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let epoch = create_epoch_fixture(&mut ctx, 1);
    open_epoch(&mut ctx, &epoch, 10_000_000);
    ctx.svm.warp_to_slot(2);
    finalize_epoch(&mut ctx, &epoch, 100);
    let claim = create_claim(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(50, 50, 50, 50),
    );

    assert!(claim_reward_result(
        &mut ctx,
        &epoch,
        claim,
        &epoch.user_two,
        epoch.user_two_token.pubkey(),
    )
    .is_err());

    claim_reward(
        &mut ctx,
        &epoch,
        claim,
        &epoch.user_one,
        epoch.user_one_token.pubkey(),
    );
    assert!(claim_reward_result(
        &mut ctx,
        &epoch,
        claim,
        &epoch.user_one,
        epoch.user_one_token.pubkey(),
    )
    .is_err());
}
