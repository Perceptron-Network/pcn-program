#[path = "harness.rs"]
mod harness;

use harness::*;

#[test]
fn update_config_requires_admin_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };

    let non_admin = solana_keypair::Keypair::new();
    let result = update_config_result(
        &mut ctx,
        &non_admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: Some(9),
            curve: None,
        },
    );
    assert!(result.is_err());

    let admin = clone_keypair(&ctx.admin);
    let result = update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: Some(9),
            curve: None,
        },
    );
    assert!(result.is_ok());

    let config: pcn_program::Config = get_anchor_account(&ctx.svm, &ctx.config);
    assert_eq!(config.claim_window_slots, 9);
}

#[test]
fn update_config_rejects_invalid_curve_and_supply_regression_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let admin = clone_keypair(&ctx.admin);

    let mut invalid_curve = ctx.curve;
    invalid_curve.saturation_units = 0;
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: Some(invalid_curve),
        },
    )
    .is_err());

    let epoch = create_epoch_fixture(&mut ctx, 1);
    open_epoch(&mut ctx, &epoch, 10_000_000);
    ctx.svm.warp_to_slot(2);
    finalize_epoch(&mut ctx, &epoch, 100);

    let mut lower_supply = ctx.curve;
    lower_supply.max_supply = 1;
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: Some(lower_supply),
        },
    )
    .is_err());
}

fn clone_keypair(keypair: &solana_keypair::Keypair) -> solana_keypair::Keypair {
    solana_keypair::Keypair::try_from(keypair.to_bytes().as_slice()).unwrap()
}
