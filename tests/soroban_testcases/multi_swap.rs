// SPDX-License-Identifier: Apache-2.0

use crate::SorobanEnv;
use soroban_sdk::{map, testutils::Address as _, Address, IntoVal, Map, Symbol, Val, Vec};

const MULTI_SWAP_SRC: &str = r#"
contract multi_swap {
    struct SwapSpec {
        address addr;
        int128 amount;
        int128 min_recv;
    }

    uint64 public swaps;
    mapping(address => uint64) public hits;

    function multi_swap_orders(
        address token_a,
        address token_b,
        SwapSpec[] memory swaps_a,
        SwapSpec[] memory swaps_b
    ) public {
        token_a;
        token_b;

        uint64[] memory matched_b = new uint64[](swaps_b.length);

        for (uint64 i = 0; i < swaps_a.length; i++) {
            for (uint64 j = 0; j < swaps_b.length; j++) {
                if (
                    matched_b[j] == 0 &&
                    swaps_a[i].amount >= swaps_b[j].min_recv &&
                    swaps_a[i].min_recv <= swaps_b[j].amount
                ) {
                    swaps += 1;
                    hits[swaps_a[i].addr] += 1;
                    hits[swaps_b[j].addr] += 1;
                    matched_b[j] = 1;
                    break;
                }
            }
        }
    }
}
"#;

fn build_swap_spec(runtime: &SorobanEnv, addr: &Address, amount: i128, min_recv: i128) -> Val {
    let spec: Map<Symbol, Val> = map![
        &runtime.env,
        (Symbol::new(&runtime.env, "addr"), addr.clone().into_val(&runtime.env)),
        (Symbol::new(&runtime.env, "amount"), amount.into_val(&runtime.env)),
        (Symbol::new(&runtime.env, "min_recv"), min_recv.into_val(&runtime.env))
    ];

    spec.into_val(&runtime.env)
}

fn assert_u64(runtime: &SorobanEnv, contract: &Address, function_name: &str, expected: u64) {
    let value = runtime.invoke_contract(contract, function_name, vec![]);
    let expected: Val = expected.into_val(&runtime.env);

    assert!(expected.shallow_eq(&value));
}

fn assert_hits(runtime: &SorobanEnv, contract: &Address, owner: &Address, expected: u64) {
    let value = runtime.invoke_contract(contract, "hits", vec![owner.clone().into_val(&runtime.env)]);
    let expected: Val = expected.into_val(&runtime.env);

    assert!(expected.shallow_eq(&value));
}

#[test]
fn multi_swap_matches_multiple_pairs() {
    let mut runtime = SorobanEnv::new();
    let multi = runtime.deploy_contract(MULTI_SWAP_SRC);

    let token_a = Address::generate(&runtime.env);
    let token_b = Address::generate(&runtime.env);
    let a1 = Address::generate(&runtime.env);
    let a2 = Address::generate(&runtime.env);
    let b1 = Address::generate(&runtime.env);
    let b2 = Address::generate(&runtime.env);

    let swaps_a: Vec<Val> = soroban_sdk::vec![
        &runtime.env,
        build_swap_spec(&runtime, &a1, 40, 30),
        build_swap_spec(&runtime, &a2, 25, 18)
    ];
    let swaps_b: Vec<Val> = soroban_sdk::vec![
        &runtime.env,
        build_swap_spec(&runtime, &b1, 50, 35),
        build_swap_spec(&runtime, &b2, 30, 20)
    ];

    runtime.invoke_contract(
        &multi,
        "multi_swap_orders",
        vec![
            token_a.into_val(&runtime.env),
            token_b.into_val(&runtime.env),
            swaps_a.into_val(&runtime.env),
            swaps_b.into_val(&runtime.env),
        ],
    );

    assert_u64(&runtime, &multi, "swaps", 2);
    assert_hits(&runtime, &multi, &a1, 1);
    assert_hits(&runtime, &multi, &a2, 1);
    assert_hits(&runtime, &multi, &b1, 1);
    assert_hits(&runtime, &multi, &b2, 1);
}

#[test]
fn multi_swap_does_not_reuse_matched_counterparty() {
    let mut runtime = SorobanEnv::new();
    let multi = runtime.deploy_contract(MULTI_SWAP_SRC);

    let token_a = Address::generate(&runtime.env);
    let token_b = Address::generate(&runtime.env);
    let a1 = Address::generate(&runtime.env);
    let a2 = Address::generate(&runtime.env);
    let b1 = Address::generate(&runtime.env);

    let swaps_a: Vec<Val> = soroban_sdk::vec![
        &runtime.env,
        build_swap_spec(&runtime, &a1, 40, 30),
        build_swap_spec(&runtime, &a2, 25, 18)
    ];
    let swaps_b: Vec<Val> = soroban_sdk::vec![
        &runtime.env,
        build_swap_spec(&runtime, &b1, 50, 35)
    ];

    runtime.invoke_contract(
        &multi,
        "multi_swap_orders",
        vec![
            token_a.into_val(&runtime.env),
            token_b.into_val(&runtime.env),
            swaps_a.into_val(&runtime.env),
            swaps_b.into_val(&runtime.env),
        ],
    );

    assert_u64(&runtime, &multi, "swaps", 1);
    assert_hits(&runtime, &multi, &a1, 1);
    assert_hits(&runtime, &multi, &a2, 0);
    assert_hits(&runtime, &multi, &b1, 1);
}
