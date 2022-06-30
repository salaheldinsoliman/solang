use crate::build_solidity;
use parity_scale_codec::{Decode, Encode};

#[test]

fn array_boundary_check() {
    //struct Args (Vec<i32>, usize, usize);
    //struct DynamicArray([u8 ;3]);
    #[derive(Encode, Decode)]
    struct SetArg(u32);

    #[derive(Encode, Decode)]
    struct BooleanArg(bool);

    let mut contract = build_solidity(
        r#"
        contract Array_bound_Test {
            function array_bound () public pure {
                    uint256[] a = new uint256[](10);
                    uint256 sesa = 0;
                    if (1>2) {
                        a.push(5);}
                        else {
                        a.push(1);
                        }
        
                        for (uint256 i = 0; i < a.length; i++) {
                            sesa = sesa + a[10];
                        }
        
                        assert (sesa ==11);
                        
                        
                }}
            

        "#,
    );
    //runs correctly

    contract.function("array_bound", Vec::new());

    let mut contract = build_solidity(
        r#"
    contract Array_bound_Test {
        function array_bound ( uint32 size32) public {
                
                uint256[] c = new uint256[] (size32);
                uint256[] d = new uint256[] (20);
                uint32 sesa = c.length + d.length;
                    

                assert (sesa == 31 );
            }}
        
        

    "#,
    );

    contract.function("array_bound", SetArg(11).encode());

    let mut contract = build_solidity(
        r#"
    contract c {
        function test(bool cond) public returns (uint32) {
                bool[] b  = new bool[](100);
                if (cond) {
                    b.push(true);
               }
              
               assert (b.length == 101);

               if (cond) {
                b.pop();
                b.pop();
               }

               assert(b.length == 99);
               return b.length;
        }
 }
        
    "#,
    );

    contract.function("test", BooleanArg(true).encode());

    let mut contract = build_solidity(
        r#"
        contract c {
            function test_for_loop() public {
                uint256[] a = new uint256[] (20);
                a. push(1);
                uint sesa = 0;
                    
                    for (uint i =0 ; i< a.length;  i++ ){
                    sesa = sesa + a[20];
                    }
                    
                    assert (sesa == 21);
                    }
                    }  
    "#,
    );

    contract.function("test_for_loop", Vec::new());
}
