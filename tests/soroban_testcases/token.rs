// SPDX-License-Identifier: Apache-2.0

use crate::build_solidity;
use soroban_sdk::{testutils::Address as _, Address, IntoVal, Val};

#[test]
fn token_end_to_end_test() {
    let runtime = build_solidity(
        r#"
        // Solidity source included above
        contract token {
            // METADATA
            address public admin = address"GCKTMAQHLCUZHEGVW4B6N4T5ZZQWLDXACNLYKYVJI5OL6NO77NZVJQXJ";
            uint32 public decimals = 18;
            string public name = "SolangToken";
            string public symbol = "SLG";
            mapping(address => int128) public balances;
            mapping(address => mapping(address => int128)) public allowances;

            function mint(address to, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                admin.requireAuth();
                setBalance(to, balance(to) + amount);
            }

            function approve(address owner, address spender, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                owner.requireAuth();
                allowances[owner][spender] = amount;
            }

            function transfer(address from, address to, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                from.requireAuth();
                require(balance(from) >= amount, "Insufficient balance");
                setBalance(from, balance(from) - amount);
                setBalance(to, balance(to) + amount);
            }

            function transfer_from(address spender, address from, address to, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                spender.requireAuth();
                require(balance(from) >= amount, "Insufficient balance");
                require(allowance(from, spender) >= amount, "Insufficient allowance");
                setBalance(from, balance(from) - amount);
                setBalance(to, balance(to) + amount);
                allowances[from][spender] -= amount;
            }

            function burn(address from, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                require(balance(from) >= amount, "Insufficient balance");
                from.requireAuth();
                setBalance(from, balance(from) - amount);
            }

            function burn_from(address spender, address from, int128 amount) public {
                require(amount >= 0, "Amount must be non-negative");
                spender.requireAuth();
                require(balance(from) >= amount, "Insufficient balance");
                require(allowance(from, spender) >= amount, "Insufficient allowance");
                setBalance(from, balance(from) - amount);
                allowances[from][spender] -= amount;
            }

            function setBalance(address addr, int128 amount) internal {
                require(amount >= 0, "Negative balance not allowed");
                balances[addr] = amount;
            }

            function balance(address addr) public view returns (int128) {
                return balances[addr];
            }

            function allowance(address owner, address spender) public view returns (int128) {
                return allowances[owner][spender];
            }
        }
        "#,
        |_| {},
    );

    runtime.env.mock_all_auths();

    let addr = runtime.contracts.last().unwrap();
    let user1 = Address::generate(&runtime.env);
    let user2 = Address::generate(&runtime.env);
    let user3 = Address::generate(&runtime.env);

    runtime.invoke_contract(
        addr,
        "mint",
        vec![
            user1.clone().into_val(&runtime.env),
            100_i128.into_val(&runtime.env),
        ],
    );

    runtime.invoke_contract(
        addr,
        "transfer",
        vec![
            user1.clone().into_val(&runtime.env),
            user2.clone().into_val(&runtime.env),
            25_i128.into_val(&runtime.env),
        ],
    );

    // print balances after transfer
    let bal1 = runtime.invoke_contract(addr, "balance", vec![user1.clone().into_val(&runtime.env)]);
    let bal2 = runtime.invoke_contract(addr, "balance", vec![user2.clone().into_val(&runtime.env)]);
    println!("User1 Balance after transfer: {:?}", bal1);
    println!("User2 Balance after transfer: {:?}", bal2);

    runtime.invoke_contract(
        addr,
        "approve",
        vec![
            user1.clone().into_val(&runtime.env),
            user3.clone().into_val(&runtime.env),
            30_i128.into_val(&runtime.env),
        ],
    );

    runtime.invoke_contract(
        addr,
        "transfer_from",
        vec![
            user3.clone().into_val(&runtime.env),
            user1.clone().into_val(&runtime.env),
            user3.clone().into_val(&runtime.env),
            10_i128.into_val(&runtime.env),
        ],
    );

    // print balances after transfer_from
    let bal1_after =
        runtime.invoke_contract(addr, "balance", vec![user1.clone().into_val(&runtime.env)]);
    let bal2_after =
        runtime.invoke_contract(addr, "balance", vec![user2.clone().into_val(&runtime.env)]);
    let bal3_after =
        runtime.invoke_contract(addr, "balance", vec![user3.clone().into_val(&runtime.env)]);

    println!("User1 Balance after transfer_from: {:?}", bal1_after);
    println!("User2 Balance after transfer_from: {:?}", bal2_after);
    println!("User3 Balance after transfer_from: {:?}", bal3_after);

    runtime.invoke_contract(
        addr,
        "burn",
        vec![
            user2.clone().into_val(&runtime.env),
            5_i128.into_val(&runtime.env),
        ],
    );

    runtime.invoke_contract(
        addr,
        "burn_from",
        vec![
            user3.clone().into_val(&runtime.env),
            user1.clone().into_val(&runtime.env),
            15_i128.into_val(&runtime.env),
        ],
    );

    let b1 = runtime.invoke_contract(addr, "balance", vec![user1.into_val(&runtime.env)]);
    let b2 = runtime.invoke_contract(addr, "balance", vec![user2.into_val(&runtime.env)]);
    let b3 = runtime.invoke_contract(addr, "balance", vec![user3.into_val(&runtime.env)]);

    println!("User1 Balance: {:?}", b1);
    println!("User2 Balance: {:?}", b2);
    println!("User3 Balance: {:?}", b3);

    // balances should be:
    // User1: 50
    // User2: 20
    // User3: 10

    let expected_user1: Val = 50_i128.into_val(&runtime.env);
    let expected_user2: Val = 20_i128.into_val(&runtime.env);
    let expected_user3: Val = 10_i128.into_val(&runtime.env);

    assert!(expected_user1.shallow_eq(&b1));
    assert!(expected_user2.shallow_eq(&b2));
    assert!(expected_user3.shallow_eq(&b3));
}
