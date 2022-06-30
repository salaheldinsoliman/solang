// RUN: --target substrate --emit cfg

contract Array_bound_Test {
    // BEGIN-CHECK: Array_bound_Test::Array_bound_Test::function::array_bound__uint256:
    function array_bound(
        uint256[] b,
        uint256 size,
        uint32 size32
    ) public pure returns (uint256) {
        // CHECK: ty:uint32 %array_size.temp.14 = (trunc uint32 (arg #1))
        uint256[] a = new uint256[](size);

        // CHECK: ty:uint32 %array_size.temp.15 = (arg #2)
        uint256[] c = new uint256[](size32);

        // CHECK: ty:uint32 %array_size.temp.16 = uint32 20
        uint256[] d = new uint256[](20);

        // CHECK: ty:uint32 %array_size.temp.14 = (%array_size.temp.14 + uint32 1)
        a.push();

        // CHECK: ty:uint32 %array_size.temp.15 = ((arg #2) - uint32 1)
        c.pop();

        // CHECK: ty:uint32 %array_size.temp.16 = uint32 21
        d.push();

        // CHECK: return (zext uint256 (((%array_size.temp.14 + (builtin ArrayLength ((arg #0)))) + ((arg #2) - uint32 1)) + uint32 21))
        return a.length + b.length + c.length + d.length;
    }

    // BEGIN-CHECK: function Array_bound_Test::Array_bound_Test::function::test__bool
    function test(bool cond) public returns (uint32) {
        bool[] b = new bool[](160);

        if (cond) {
            // CHECK: ty:uint32 %array_size.temp.20 = uint32 161
            b.push(true);
        }

        //CHECK : return %array_size.temp.20
        return b.length;
    }

    // BEGIN_CHECK : function Array_bound_Test::Array_bound_Test::function::test_for_loop
    function test_for_loop() public {
        uint256[] a = new uint256[](20);
        a.push(1);
        uint256 sesa = 0;

        // CHECK : branchcond (unsigned less %i < uint256 21), block1, block4
        for (uint256 i = 0; i < a.length; i++) {
            // CHECK : branchcond (uint32 20 >= uint32 21), block5, block6
            sesa = sesa + a[20];
        }

        assert(sesa == 21);
    }
}
