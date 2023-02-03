// SPDX-License-Identifier: Apache-2.0

use crate::build_solidity_with_options;
use parity_scale_codec::Encode;

#[test]
fn errors() {
    let mut runtime = build_solidity_with_options(
        r#"contract Printer {
        bytes b = hex"0000_00fa";
        uint256[] arr;
        child public c;
        child public c2;
        callee public cal;
    
        constructor() public payable {}
    
        function print_test(int8 num) public returns (int8) {
            print("Hello world!");
            return num;
        }
    
        function math_overflow(int8 num) public returns (int8) {
            int8 ovf = num + 120;
            return ovf;
        }
    
        function require_test(int8 num) public returns (int8) {
            require(num > 10, "sesa");
            return 0;
        }
    
        // assert failure
        function assert_test(int8 num) public returns (int8) {
            assert(num > 10);
            return 0;
        }
    
        // set storage index out of bounds
        function set_storage_bytes() public returns (bytes) {
            bytes sesa = new bytes(1);
            b[5] = sesa[0];
            return sesa;
        }
    
        // storage array index out of bounds
        function get_storage_bytes() public returns (bytes) {
            bytes sesa = new bytes(1);
            sesa[0] = b[5];
            return sesa;
        }
    
        // value transfer failure
        function transfer_abort() public {
            address a = address(0);
            payable(a).transfer(10);
        }
    
        //  pop from empty storage array
        function pop_empty_storage() public {
            arr.pop();
        }
    
        // external call failed
        function call_ext() public {
            //cal = new callee();
            cal.callee_func{gas: 1e15}();
        }
    
        // contract creation failed (contract was deplyed with no value)
        function create_child() public {
            c = new child{value: 900e15, salt:2}();
            c2 = new child{value: 900e15, salt:2}();
            uint128 x = address(this).balance;
            //print("sesa");
            print("x = {}".format(x));
            
        }
    
        // non payable function recieved value
        function dont_pay_me() public {}
    
        function pay_me() public payable {
            print("PAYED");
            uint128 x = address(this).balance;
            //print("sesa");
            print("x = {}".format(x));
            
        }
    
        function i_will_revert() public {
            revert();
        }
    
        function write_integer_failure(uint8 buf_size) public {
            bytes smol_buf = new bytes(buf_size);
            smol_buf.writeUint32LE(350, 20);
        }
    
        function write_bytes_failure(uint8 buf_size) public {
            bytes data = new bytes(10);
            bytes smol_buf = new bytes(buf_size);
            smol_buf.writeBytes(data, 0);
        }
    
        function read_integer_failure(uint32 offset) public {
            bytes smol_buf = new bytes(1);
            smol_buf.readUint16LE(offset);
        }
    
        // truncate type overflow
        function trunc_failure(uint128 input) public returns (uint256) {
            uint256[] a = new uint256[](input);
            return a[0];
        }
    
        function out_of_bounds(uint8 input) public returns (uint256) {
            uint256[] a = new uint256[](input);
            return a[20];
        }
    
        function invalid_instruction() public {
            assembly {
                invalid()
            }
        }
    
        function byte_cast_failure(uint8 num) public returns (bytes) {
            bytes smol_buf = new bytes(num);
    
            //bytes32 b32 = new bytes(num);
            bytes32 b32 = bytes32(smol_buf);
            return b32;
        }
    }
    
    contract callee {
        constructor() {}
    
        function callee_func() public {
            revert();
        }
    }
    
    contract child {
        constructor() {}
    
        function say_my_name() public pure returns (string memory) {
            print("say_my_name");
            return "child";
        }
    }
    
    "#,
        true,
        false,
        true,
    );

    runtime.function_expect_failure("write_bytes_failure", 9u8.encode());
    assert_eq!(
        runtime.printbuf,
        "data does not fit into buffer in  file: 1, line: 95,21"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("math_overflow", 10u8.encode());
    assert_eq!(runtime.printbuf, "math overflow in  file: 1, line: 16,23");

    runtime.printbuf.clear();
    runtime.function_expect_failure("require_test", 9u8.encode());
    assert_eq!(runtime.printbuf, "sesa");

    runtime.printbuf.clear();
    runtime.function_expect_failure("assert_test", 9u8.encode());
    assert_eq!(runtime.printbuf, "assert failure in  file: 1, line: 27,19");

    runtime.printbuf.clear();
    runtime.function_expect_failure("set_storage_bytes", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "set storage index out of bounds in  file: 1, line: 34,14"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("get_storage_bytes", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "storage array index out of bounds in  file: 1, line: 41,22"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("transfer_abort", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "value transfer failure in  file: 1, line: 48,32"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("pop_empty_storage", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "pop from empty storage array in  file: 1, line: 53,16"
    );

    runtime.vm.value = 3500;
    runtime.constructor(0, Vec::new());

    runtime.printbuf.clear();
    runtime.vm.value = 0;
    runtime.function_expect_failure("create_child", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "contract creation failed in  file: 1, line: 65,17"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("i_will_revert", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "revert encountered in  file: 1, line: 84,12"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("write_integer_failure", 1u8.encode());
    assert_eq!(
        runtime.printbuf,
        "integer too large to write in buffer in  file: 1, line: 89,21"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("invalid_instruction", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "reached invalid instruction in  file: 1, line: 116,16"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("out_of_bounds", 19u8.encode());
    assert_eq!(
        runtime.printbuf,
        "array out of bounds in  file: 1, line: 111,19"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("trunc_failure", u128::MAX.encode());
    assert_eq!(
        runtime.printbuf,
        "truncate type overflow in  file: 1, line: 105,40"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("byte_cast_failure", 33u8.encode());
    assert_eq!(
        runtime.printbuf,
        "bytes cast error in  file: 1, line: 124,26"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("read_integer_failure", 2u32.encode());
    assert_eq!(
        runtime.printbuf,
        "read integer out of bounds in  file: 1, line: 100,21"
    );

    runtime.printbuf.clear();
    runtime.function_expect_failure("call_ext", Vec::new());
    assert_eq!(
        runtime.printbuf,
        "external call failed in  file: 1, line: 59,12"
    );

    runtime.printbuf.clear();
    runtime.vm.value = 1;
    runtime.function_expect_failure("dont_pay_me", Vec::new());
    assert_eq!(runtime.printbuf, "non payable function recieved value");
}

/*fn debug_buff () {


    let mut runtime = build_solidity_with_options(src, math_overflow_flag, log_api_return_codes, log_runtime_errors)
}*/
