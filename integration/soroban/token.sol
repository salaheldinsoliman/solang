contract token {

    // METADATA
    address public admin = address"GCKTMAQHLCUZHEGVW4B6N4T5ZZQWLDXACNLYKYVJI5OL6NO77NZVJQXJ";
    uint32  public decimals = 18;
    string public name = "SolangToken";
    string public symbol = "SLG";


    // STORAGE
    mapping(address => int128) public balances;
    mapping(address => mapping(address => int128)) public allowances;



    function mint(address to, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // require auth from admin
        admin.requireAuth();

        // call setBalance
        setBalance(to, balance(to) + amount);
    }


    // set the allowance for an address to spend another address's tokens
    function approve(address owner, address spender, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // require auth from owner
        owner.requireAuth();

        allowances[owner][spender] = amount;
    }

    // transfer tokens from one address to another
    function transfer(address from, address to, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // require auth from from
        from.requireAuth();

        // check if from has enough balance
        require(balance(from) >= amount, "Insufficient balance");

        // update balances
        setBalance(from, balance(from) - amount);
        setBalance(to, balance(to) + amount);
    }

    // transfer tokens from one address to another using allowance
    function transfer_from(address spender, address from, address to, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // require auth from spender
        spender.requireAuth();

        // check if from has enough balance
        require(balance(from) >= amount, "Insufficient balance");

        // check if spender has enough allowance
        require(allowance(from, spender) >= amount, "Insufficient allowance");

        // update balances and allowances
        setBalance(from, balance(from) - amount);
        setBalance(to, balance(to) + amount);
        allowances[from][spender] -= amount;
    }

    // burn tokens from an address
    function burn(address from, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // check if from has enough balance
        require(balance(from) >= amount, "Insufficient balance");

        // require auth from from
        from.requireAuth();

        // update balance
        setBalance(from, balance(from) - amount);
    }

    // burn tokens from an address using allowance
    function burn_from(address spender, address from, int128 amount) public {
        // check non negative amount
        require(amount >= 0, "Amount must be non-negative");

        // require auth from spender
        spender.requireAuth();

        // check if from has enough balance
        require(balance(from) >= amount, "Insufficient balance");

        // check if spender has enough allowance
        require(allowance(from, spender) >= amount, "Insufficient allowance");

        // update balances and allowances
        setBalance(from, balance(from) - amount);
        allowances[from][spender] -= amount;
    }



    ///////////// HELPERS /////////////

    
    // set the balance of an address
    function setBalance(address addr, int128 amount) internal {
        balances[addr] = amount;
    }

    // get the balance of an address
    function balance(address addr) public view returns (int128) {
        return balances[addr];
    }

    // get the allowance for an address to spend another address's tokens
    function allowance(address owner, address spender) public view returns (int128) {
        return allowances[owner][spender];
    }

}