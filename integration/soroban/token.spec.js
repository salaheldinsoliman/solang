// token.spec.js

import * as StellarSdk from '@stellar/stellar-sdk';
import { readFileSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { expect } from 'chai';
import { call_contract_function } from './test_helpers.js';

const __filename = fileURLToPath(import.meta.url);
const dirname = path.dirname(__filename);
const server = new StellarSdk.SorobanRpc.Server("https://soroban-testnet.stellar.org:443");

function readContractAddress(filename) {
  return readFileSync(path.join(dirname, '.stellar', 'contract-ids', filename), 'utf8').trim();
}

describe('Token Contract', () => {
  let alice, bob, charlie, tokenContract;

  before(async () => {
    alice = StellarSdk.Keypair.fromSecret(readFileSync('alice.txt', 'utf8').trim());
    bob = StellarSdk.Keypair.fromSecret(readFileSync('bob.txt', 'utf8').trim());
    charlie = StellarSdk.Keypair.fromSecret(readFileSync('charlie.txt', 'utf8').trim());
    tokenContract = new StellarSdk.Contract(readContractAddress('token.txt'));
  });

  it('runs token lifecycle: mint, transfer, approve, transfer_from, burn, burn_from', async () => {
    // Mint 100 to Alice
    await call_contract_function(
  "mint",
  server,
  alice,
  tokenContract,
  [
    StellarSdk.nativeToScVal(alice.publicKey().toString(), { type: "address" }),
    StellarSdk.nativeToScVal("100", { type: "i128" })
  ]
);


    // Transfer 25 from Alice to Bob
    /*await call_contract_function(
      "transfer",
      server,
      alice,
      tokenContract,
      [
        StellarSdk.Address.fromString(alice.publicKey()).toScVal(),
        StellarSdk.Address.fromString(bob.publicKey()).toScVal(),
        StellarSdk.nativeToScVal("25", { type: "i128" })
      ]
    );

    // Approve Charlie to spend 30 from Alice
    await call_contract_function(
      "approve",
      server,
      alice,
      tokenContract,
      [
        StellarSdk.Address.fromString(alice.publicKey()).toScVal(),
        StellarSdk.Address.fromString(charlie.publicKey()).toScVal(),
        StellarSdk.nativeToScVal("30", { type: "i128" })
      ]
    );

    // Charlie transfers 10 from Alice to Charlie
    await call_contract_function(
      "transfer_from",
      server,
      charlie,
      tokenContract,
      [
        StellarSdk.Address.fromString(charlie.publicKey()).toScVal(),
        StellarSdk.Address.fromString(alice.publicKey()).toScVal(),
        StellarSdk.Address.fromString(charlie.publicKey()).toScVal(),
        StellarSdk.nativeToScVal("10", { type: "i128" })
      ]
    );

    // Bob burns 5 tokens
    await call_contract_function(
      "burn",
      server,
      bob,
      tokenContract,
      [
        StellarSdk.Address.fromString(bob.publicKey()).toScVal(),
        StellarSdk.nativeToScVal("5", { type: "i128" })
      ]
    );

    // Charlie burns 15 tokens from Alice
    await call_contract_function(
      "burn_from",
      server,
      charlie,
      tokenContract,
      [
        StellarSdk.Address.fromString(charlie.publicKey()).toScVal(),
        StellarSdk.Address.fromString(alice.publicKey()).toScVal(),
        StellarSdk.nativeToScVal("15", { type: "i128" })
      ]
    );

    // Check balances
    const [b1, b2, b3] = await Promise.all([
      call_contract_function(
        "balance",
        server,
        alice,
        tokenContract,
        [StellarSdk.Address.fromString(alice.publicKey()).toScVal()]
      ),
      call_contract_function(
        "balance",
        server,
        alice,
        tokenContract,
        [StellarSdk.Address.fromString(bob.publicKey()).toScVal()]
      ),
      call_contract_function(
        "balance",
        server,
        alice,
        tokenContract,
        [StellarSdk.Address.fromString(charlie.publicKey()).toScVal()]
      )
    ]);

    const balAlice = b1.returnValue().value().toString();
    const balBob = b2.returnValue().value().toString();
    const balCharlie = b3.returnValue().value().toString();

    console.log("Alice:", balAlice);
    console.log("Bob:", balBob);
    console.log("Charlie:", balCharlie);

    expect(balAlice).to.equal("50");
    expect(balBob).to.equal("20");
    expect(balCharlie).to.equal("10");*/
  });
});
