// SPDX-License-Identifier: Apache-2.0

import { Connection, Keypair, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction } from '@solana/web3.js';
import expect from 'expect';
import { Contract, TransactionError } from '@solana/solidity';
import { loadContract } from './setup';
import fs from 'fs';

describe('DebugBuffer', function () {
    this.timeout(150000);

    let contract: Contract;
    let payer: Keypair;
    let connection: Connection;

    before(async function () {
        ({ contract, payer, connection } = await loadContract('Printer'));
    });

    it('Prints runtime errors', async function () {
  
        try {
            let res = await contract.functions.set_storage_bytes({ simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("set storage index out of bounds in  file: 1, line: 42,10")
            }
        }

        try {
            let res = await contract.functions.get_storage_bytes({ simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("storage array index out of bounds in  file: 1, line: 49,18")
            }
        }

        try {
            let res = await contract.functions.pop_empty_storage({ simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("pop from empty storage array in  file: 1, line: 61,8")
            }
        }

        try {
            let res = await contract.functions.invalid_instruction({ simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("reached invalid instruction in  file: 1, line: 113,12")
            }
        }

        try {
            let res = await contract.functions.byte_cast_failure(33, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("bytes cast error in  file: 1, line: 121,22")
            }
        }

        try {
            let res = await contract.functions.i_will_revert({ simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("revert encountered in  file: 1, line: 81,8")
            }
        }

        try {
            let res = await contract.functions.assert_test(9, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("assert failure in  file: 1, line: 35,15")
            }
        }

        try {
            let res = await contract.functions.write_integer_failure(1, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("integer too large to write in buffer in  file: 1, line: 86,17")
            }
        }

        try {
            let res = await contract.functions.write_bytes_failure(9, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("data does not fit into buffer in  file: 1, line: 92,17")
            }
        }


        try {
            let res = await contract.functions.read_integer_failure(2, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("read integer out of bounds in  file: 1, line: 97,17")
            }
        }


        try {
            let res = await contract.functions.out_of_bounds(19, { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("array out of bounds in  file: 1, line: 108,15")
            }
        }


        try {
            let res = await contract.functions.trunc_failure(BigInt("999999999999999999999999"), { simulate: true });
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("truncate type overflow in  file: 1, line: 102,36")
            }
        }

        let child_program = new PublicKey("Crea1hXZv5Snuvs38GW2SJ1vJQ2Z5uBavUnwPwpiaDiQ");
        let child = Keypair.generate();

        let ress = await contract.functions.create_child(child.publicKey. toBytes(), payer.publicKey.toBytes(), {
            accounts: [child_program],
            writableAccounts: [child.publicKey],
            signers: [child, payer],
            //simulate: true
        });
        try {
            let res = await contract.functions.create_child(child.publicKey. toBytes(), payer.publicKey.toBytes(), {
                accounts: [child_program],
                //writableAccounts: [child.publicKey],
                signers: [  payer],
                simulate: true
            });

            console.log (res)
        } catch (e) {
            expect(e).toBeInstanceOf(TransactionError);
            if (e instanceof TransactionError) {
                expect(e.message).toEqual("contract creation failed in  file: 1, line: 71,12")
            }
        }

        });
        
    });

