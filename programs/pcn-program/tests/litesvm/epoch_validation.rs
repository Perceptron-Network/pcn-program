#[path = "harness.rs"]
mod harness;

use harness::*;

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
    assert!(finalize_epoch_result(&mut ctx, &epoch_success, &oracle, 100).is_ok());
}
