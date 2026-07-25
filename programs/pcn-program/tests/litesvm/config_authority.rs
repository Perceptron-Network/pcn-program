#[path = "harness.rs"]
mod harness;

use harness::*;
use solana_signer::Signer;

#[test]
fn initialize_config_creates_program_owned_sol_reserve_in_litesvm() {
    let Some(ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };

    let reserve = ctx.svm.get_account(&ctx.sol_reserve).unwrap();
    assert_eq!(reserve.owner, pcn_program::id());
    assert_eq!(reserve.data.len(), pcn_program::SolReserve::LEN);
    let _: pcn_program::SolReserve = get_anchor_account(&ctx.svm, &ctx.sol_reserve);
}

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
            performance_weights: None,
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
            performance_weights: None,
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
            performance_weights: None,
        },
    )
    .is_err());

    invalid_curve = ctx.curve;
    invalid_curve.emission_multiplier_ppm = 0;
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: Some(invalid_curve),
            performance_weights: None,
        },
    )
    .is_err());

    invalid_curve.emission_multiplier_ppm = pcn_program::EMISSION_MULTIPLIER_PPM_SCALE + 1;
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: Some(invalid_curve),
            performance_weights: None,
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
            performance_weights: None,
        },
    )
    .is_err());
}

#[test]
fn admin_can_update_emission_multiplier_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let admin = clone_keypair(&ctx.admin);
    let mut reduced_curve = ctx.curve;
    reduced_curve.emission_multiplier_ppm = 500_000;

    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: Some(reduced_curve),
            performance_weights: None,
        },
    )
    .is_ok());

    let config: pcn_program::Config = get_anchor_account(&ctx.svm, &ctx.config);
    assert_eq!(config.curve.emission_multiplier_ppm, 500_000);
}

#[test]
fn admin_can_update_valid_performance_weights_and_invalid_sum_is_rejected_in_litesvm() {
    let Some(mut ctx) = setup_pcn_litesvm() else {
        eprintln!("skipping LiteSVM test; run `anchor test` first");
        return;
    };
    let admin = clone_keypair(&ctx.admin);
    let valid = pcn_program::PerformanceWeights {
        uptime_ppm: 100_000,
        bandwidth_ppm: 400_000,
        fulfilment_rate_ppm: 200_000,
        quest_score_ppm: 300_000,
    };
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: None,
            performance_weights: Some(valid),
        },
    )
    .is_ok());

    let mut invalid = valid;
    invalid.quest_score_ppm -= 1;
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: None,
            performance_weights: Some(invalid),
        },
    )
    .is_err());

    let config: pcn_program::Config = get_anchor_account(&ctx.svm, &ctx.config);
    assert_eq!(config.performance_weights, valid);

    let epoch = create_epoch_fixture(&mut ctx, 1);
    open_epoch(&mut ctx, &epoch, 10_000_000);
    assert!(update_config_result(
        &mut ctx,
        &admin,
        pcn_program::UpdateConfigArgs {
            oracle: None,
            claim_window_slots: None,
            curve: None,
            performance_weights: Some(equal_performance_weights()),
        },
    )
    .is_ok());
    ctx.svm.warp_to_slot(2);
    finalize_epoch(&mut ctx, &epoch, 100_000);
    let claim = create_claim(
        &mut ctx,
        &epoch,
        epoch.user_one.pubkey(),
        performance(pcn_program::PERFORMANCE_PPM_SCALE, 0, 0, 0),
    );
    let claim: pcn_program::Claim = get_anchor_account(&ctx.svm, &claim);
    assert_eq!(claim.reward_weight, 100_000);
}

fn clone_keypair(keypair: &solana_keypair::Keypair) -> solana_keypair::Keypair {
    solana_keypair::Keypair::try_from(keypair.to_bytes().as_slice()).unwrap()
}
