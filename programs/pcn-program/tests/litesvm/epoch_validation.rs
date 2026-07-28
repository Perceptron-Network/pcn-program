#[path = "harness.rs"]
mod harness;

use anchor_lang::prelude::Pubkey;
use harness::*;
use solana_signer::Signer;

#[test]
fn open_and_finalize_require_oracle_and_valid_inputs_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };

    let epoch = create_epoch_fixture(&mut ctx, 1);
    let non_oracle = solana_keypair::Keypair::new();
    assert!(open_epoch_result(&mut ctx, &epoch, &non_oracle, 10_000_000).is_err());

    let epoch_zero_support = create_epoch_fixture(&mut ctx, 2);
    let oracle = solana_keypair::Keypair::try_from(ctx.oracle.to_bytes().as_slice()).unwrap();
    assert!(open_epoch_result(&mut ctx, &epoch_zero_support, &oracle, 0).is_err());

    open_epoch(&mut ctx, &epoch, 10_000_000);
    assert!(finalize_epoch_result(&mut ctx, &epoch, &non_oracle, 100).is_err());
    assert!(finalize_epoch_result(&mut ctx, &epoch, &oracle, 100).is_err());

    let epoch_success = create_epoch_fixture(&mut ctx, 3);
    open_epoch(&mut ctx, &epoch_success, 10_000_000);
    ctx.svm.warp_to_slot(2);
    assert!(finalize_epoch_result(&mut ctx, &epoch_success, &oracle, 0).is_err());
    ctx.svm.expire_blockhash();
    assert!(finalize_epoch_result(&mut ctx, &epoch_success, &oracle, 100).is_ok());
}

#[test]
fn finalize_refunds_only_the_original_support_funder_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let epoch = create_epoch_fixture(&mut ctx, 1);
    open_epoch(&mut ctx, &epoch, 10_000_000);
    ctx.svm.warp_to_slot(2);

    let attacker = solana_keypair::Keypair::new();
    ctx.svm.airdrop(&attacker.pubkey(), 1_000_000).unwrap();
    assert!(finalize_epoch_with_funder_result(&mut ctx, &epoch, attacker.pubkey(), 100).is_err());

    finalize_epoch(&mut ctx, &epoch, 100);
    let finalized: pcn_program::Epoch = get_anchor_account(&ctx.svm, &epoch.epoch);
    assert_eq!(finalized.support_funder, ctx.payer.pubkey());
}

#[test]
fn finalize_refunds_recorded_funder_after_ownership_changes_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let epoch = create_epoch_fixture(&mut ctx, 1);
    let funder = solana_keypair::Keypair::new();
    ctx.svm.airdrop(&funder.pubkey(), 100_000_000).unwrap();
    open_epoch_with_funder(&mut ctx, &epoch, &funder, 10_000_000);

    let mut funder_account = ctx.svm.get_account(&funder.pubkey()).unwrap();
    funder_account.owner = Pubkey::new_unique();
    ctx.svm
        .set_account(funder.pubkey(), funder_account)
        .unwrap();
    let balance_before_refund = ctx.svm.get_balance(&funder.pubkey()).unwrap();

    ctx.svm.warp_to_slot(2);
    let result = finalize_epoch_with_funder_result(&mut ctx, &epoch, funder.pubkey(), 100);
    assert!(result.is_ok(), "finalize failed: {result:?}");
    assert!(ctx.svm.get_balance(&funder.pubkey()).unwrap() > balance_before_refund);
}
