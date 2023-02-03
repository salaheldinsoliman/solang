use crate::{build_solidity_with_options, BorshToken};
use num_bigint::BigInt;

#[test]
fn runtime_errors() {
    let mut vm = build_solidity_with_options(
        r#"
contract Printer {
    bytes b = hex"0000_00fa";
    uint256[] arr;
    child public c;
    child public c2;

    constructor() {}

    function print_test(int8 num) public returns (int8) {
        print("Hello world!");

        require(num > 10, "sesa");
        assert(num > 10);

        int8 ovf = num + 120;
        print("x = {}".format(ovf));
        return ovf;
    }

    function math_overflow(int8 num) public returns (int8) {
        int8 ovf = num + 120;
        print("x = {}".format(ovf));
        return ovf;
    }

    function require_test(int256 num) public returns (int8) {
        require(num > 10, "sesa");
        return 0;
    }

    // assert failure
    function assert_test(int256 num) public returns (int8) {
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

    //  pop from empty storage array
    function pop_empty_storage() public {
        arr.pop();
    }


    // contract creation failed
    function create_child() public {
        address a = address(0);
        c = new child{address: a}();
        //c2 = new child();
        uint128 x = address(this).balance;
        //print("sesa");
        print("x = {}".format(x));
        
    }

    // non payable function recieved value
    function dont_pay_me() public {}

    function pay_me() public payable {}

    function i_will_revert() public {
        revert();
    }

    function write_integer_failure(uint256 buf_size) public {
        bytes smol_buf = new bytes(buf_size);
        smol_buf.writeUint32LE(350, 20);
    }

    function write_bytes_failure(uint256 buf_size) public {
        bytes data = new bytes(10);
        bytes smol_buf = new bytes(buf_size);
        smol_buf.writeBytes(data, 0);
    }

    function read_integer_failure(uint32 offset) public {
        bytes smol_buf = new bytes(1);
        smol_buf.readUint16LE(offset);
    }

    // truncate type overflow
    function trunc_failure(uint256 input) public returns (uint256[]) {
        uint256[] a = new uint256[](input);
        return a;
    }

    function out_of_bounds(uint256 input) public returns (uint256) {
        uint256[] a = new uint256[](input);
        return a[20];
    }

    function invalid_instruction() public {
        assembly {
            invalid()
        }
    }

    function byte_cast_failure(uint256 num) public returns (bytes) {
        bytes smol_buf = new bytes(num);

        //bytes32 b32 = new bytes(num);
        bytes32 b32 = bytes32(smol_buf);
        return b32;
    }

}

@program_id("Crea1hXZv5Snuvs38GW2SJ1vJQ2Z5uBavUnwPwpiaDiQ")
contract child {
    constructor() {}

    function say_my_name() public pure returns (string memory) {
        print("say_my_name");
        return "child";
    }
}

contract calle_contract {
    constructor() {}

    function calle_contract_func() public {
        revert();
    }
}

 "#,
        true,
        true,
    );

    vm.set_program(0);
    vm.constructor(&[]);

    let mut _res = vm.function_must_fail(
        "math_overflow",
        &[BorshToken::Int {
            width: 8,
            value: BigInt::from(10u8),
        }],
    );
    assert_eq!(vm.logs, "math overflow in  file: 1, line: 22,19");
    vm.logs.clear();

    _res = vm.function_must_fail(
        "require_test",
        &[BorshToken::Int {
            width: 256,
            value: BigInt::from(9u8),
        }],
    );
    assert_eq!(vm.logs, "sesa");
    vm.logs.clear();

    _res = vm.function_must_fail("get_storage_bytes", &[]);
    assert_eq!(
        vm.logs,
        "storage array index out of bounds in  file: 1, line: 48,18"
    );
    vm.logs.clear();

    _res = vm.function_must_fail("set_storage_bytes", &[]);
    assert_eq!(
        vm.logs,
        "set storage index out of bounds in  file: 1, line: 41,10"
    );
    vm.logs.clear();

    _res = vm.function_must_fail(
        "read_integer_failure",
        &[BorshToken::Uint {
            width: 32,
            value: BigInt::from(2u8),
        }],
    );
    assert_eq!(
        vm.logs,
        "read integer out of bounds in  file: 1, line: 91,17"
    );
    vm.logs.clear();

    _res = vm.function_must_fail(
        "trunc_failure",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(u128::MAX),
        }],
    );
    assert_eq!(vm.logs, "truncate type overflow in  file: 1, line: 96,36");
    vm.logs.clear();

    _res = vm.function_must_fail("invalid_instruction", &[]);
    assert_eq!(
        vm.logs,
        "reached invalid instruction in  file: 1, line: 107,12"
    );
    vm.logs.clear();

    _res = vm.function_must_fail("pop_empty_storage", &[]);
    assert_eq!(
        vm.logs,
        "pop from empty storage array in  file: 1, line: 54,8"
    );
    vm.logs.clear();

    _res = vm.function_must_fail(
        "write_bytes_failure",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(9u8),
        }],
    );
    assert_eq!(
        vm.logs,
        "data does not fit into buffer in  file: 1, line: 86,17"
    );
    vm.logs.clear();

    _res = vm.function_must_fail(
        "assert_test",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(9u8),
        }],
    );
    assert_eq!(vm.logs, "assert failure in  file: 1, line: 34,15");
    vm.logs.clear();

    _res = vm.function_must_fail(
        "out_of_bounds",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(19u8),
        }],
    );
    assert_eq!(vm.logs, "array out of bounds in  file: 1, line: 102,15");
    vm.logs.clear();

    _res = vm.function_must_fail(
        "write_integer_failure",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(1u8),
        }],
    );

    assert_eq!(
        vm.logs,
        "integer too large to write in buffer in  file: 1, line: 80,17"
    );
    vm.logs.clear();

    _res = vm.function_must_fail(
        "byte_cast_failure",
        &[BorshToken::Uint {
            width: 256,
            value: BigInt::from(33u8),
        }],
    );
    assert_eq!(vm.logs, "bytes cast error in  file: 1, line: 115,22");
    vm.logs.clear();

    _res = vm.function_must_fail("i_will_revert", &[]);
    assert_eq!(vm.logs, "revert encountered in  file: 1, line: 75,8");
    vm.logs.clear();

    _res = vm.function_must_fail("create_child", &[]);
    assert_eq!(vm.logs, "contract creation failed in  file: 1, line: 61,12");
    vm.logs.clear();
}
