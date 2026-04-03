// SPDX-License-Identifier: Apache-2.0

use crate::build_solidity;
use soroban_sdk::{map, IntoVal, Map, Symbol, Val, Vec};

fn build_pairs(runtime: &crate::SorobanEnv) -> (Vec<Val>, Vec<Val>) {
    let pair1: Map<Symbol, Val> = map![
        &runtime.env,
        (
            Symbol::new(&runtime.env, "a"),
            11_u64.into_val(&runtime.env)
        ),
        (
            Symbol::new(&runtime.env, "b"),
            4_u32.into_val(&runtime.env)
        )
    ];
    let pair2: Map<Symbol, Val> = map![
        &runtime.env,
        (
            Symbol::new(&runtime.env, "a"),
            5_u64.into_val(&runtime.env)
        ),
        (
            Symbol::new(&runtime.env, "b"),
            13_u32.into_val(&runtime.env)
        )
    ];

    let pairs: Vec<Val> = soroban_sdk::vec![
        &runtime.env,
        pair1.clone().into_val(&runtime.env),
        pair2.into_val(&runtime.env)
    ];
    let single_pair: Vec<Val> = soroban_sdk::vec![&runtime.env, pair1.into_val(&runtime.env)];

    (pairs, single_pair)
}

#[test]
fn struct_array_argument_memory_roundtrip() {
    let contract_src = r#"
        contract arr_struct {
            struct Pair {
                uint64 a;
                uint32 b;
            }

            function memory_first_a(Pair[] memory pairs) public pure returns (uint64) {
                Pair memory pair = pairs[0];
                return pair.a;
            }

            function memory_first_b(Pair[] memory pairs) public pure returns (uint64) {
                Pair memory pair = pairs[0];
                return uint64(pair.b);
            }
        }
    "#;

    let mut runtime = build_solidity(contract_src, |_| {});
    let addr = runtime.contracts.last().unwrap();
    let (pairs, _) = build_pairs(&runtime);

    let expected: Val = 11_u64.into_val(&runtime.env);
    let res = runtime.invoke_contract(
        addr,
        "memory_first_a",
        std::vec![pairs.clone().into_val(&runtime.env)],
    );
    assert!(expected.shallow_eq(&res));

    let addr2 = runtime.deploy_contract(contract_src);
    let expected: Val = 4_u64.into_val(&runtime.env);
    let res = runtime.invoke_contract(
        &addr2,
        "memory_first_b",
        std::vec![pairs.into_val(&runtime.env)],
    );
    assert!(expected.shallow_eq(&res));
}

#[test]
fn struct_array_argument_storage_roundtrip() {
    let contract_src = r#"
        contract arr_struct {
            struct Pair {
                uint64 a;
                uint32 b;
            }

            Pair stored_pair;
            uint64 stored_a;
            uint32 stored_b;

            function storage_first_a(Pair[] memory pairs) public returns (uint64) {
                Pair memory pair = pairs[0];
                stored_a = pair.a;

                return stored_a;
            }

            function storage_first_b(Pair[] memory pairs) public returns (uint64) {
                Pair memory pair = pairs[0];
                stored_b = pair.b;

                return uint64(stored_b);
            }

            function store_first_pair(Pair[] memory pairs) public returns (uint64) {
                stored_pair = pairs[0];

                return stored_pair.a + uint64(stored_pair.b);
            }

            function read_stored_pair() public view returns (uint64) {
                return stored_pair.a + uint64(stored_pair.b);
            }
        }
    "#;

    let mut runtime = build_solidity(contract_src, |_| {});
    let addr = runtime.contracts.last().unwrap();
    let (_, single_pair) = build_pairs(&runtime);

    let expected: Val = 11_u64.into_val(&runtime.env);
    let res = runtime.invoke_contract(
        addr,
        "storage_first_a",
        std::vec![single_pair.clone().into_val(&runtime.env)],
    );
    assert!(expected.shallow_eq(&res));

    let addr2 = runtime.deploy_contract(contract_src);
    let expected: Val = 4_u64.into_val(&runtime.env);
    let res = runtime.invoke_contract(
        &addr2,
        "storage_first_b",
        std::vec![single_pair.into_val(&runtime.env)],
    );
    assert!(expected.shallow_eq(&res));

    let addr3 = runtime.deploy_contract(contract_src);
    let expected: Val = 15_u64.into_val(&runtime.env);
    let res = runtime.invoke_contract(
        &addr3,
        "store_first_pair",
        std::vec![build_pairs(&runtime).1.into_val(&runtime.env)],
    );
    assert!(expected.shallow_eq(&res));

    let res = runtime.invoke_contract(&addr3, "read_stored_pair", std::vec![]);
    assert!(expected.shallow_eq(&res));
}
