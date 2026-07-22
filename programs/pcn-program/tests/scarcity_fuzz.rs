use pcn_program::{calculate_scarcity_cap, CurveParams};
use rand::{rngs::OsRng, RngCore};
use std::time::{Duration, Instant};

#[test]
#[ignore = "long-running OS-random scarcity fuzz target"]
fn scarcity_fuzz() {
    let seconds = std::env::var("PCN_FUZZ_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(600);
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut cases = 0_u64;

    while Instant::now() < deadline {
        let params = CurveParams {
            max_epoch_mint: nonzero(OsRng.next_u64()),
            saturation_units: nonzero(OsRng.next_u64()),
            history_minted: nonzero(OsRng.next_u64()),
            target_support_lamports_per_token: nonzero(OsRng.next_u64()),
            max_supply: nonzero(OsRng.next_u64()),
        };
        let weight = OsRng.next_u64();
        let minted = OsRng.next_u64();

        let result = calculate_scarcity_cap(params, weight, minted).unwrap();
        assert!(result <= params.max_epoch_mint);
        if weight == 0 {
            assert_eq!(result, 0);
        }

        let increased_weight =
            calculate_scarcity_cap(params, weight.saturating_add(1), minted).unwrap();
        assert!(
            increased_weight >= result,
            "weight monotonicity: params={params:?} weight={weight} minted={minted} result={result} increased={increased_weight}"
        );

        let increased_history =
            calculate_scarcity_cap(params, weight, minted.saturating_add(1)).unwrap();
        assert!(
            increased_history <= result,
            "history monotonicity: params={params:?} weight={weight} minted={minted} result={result} increased={increased_history}"
        );
        cases += 1;
    }

    eprintln!("scarcity fuzz completed {cases} cases in {seconds} seconds");
}

fn nonzero(value: u64) -> u64 {
    value.max(1)
}
