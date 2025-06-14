
import 'dotenv/config';
import { mkdirSync, readdirSync} from 'fs';
import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

console.log("###################### Initializing ########################");

// Get dirname (equivalent to the Bash version)
const __filename = fileURLToPath(import.meta.url);
const dirname = path.dirname(__filename);

// variable for later setting pinned version of stellar in "$(dirname/target/bin/stellar)"
const stellar = "stellar"

// Function to execute and log shell commands
function exe(command) {
  console.log(command);
  execSync(command, { stdio: 'inherit' });
}

// Generate testnet keys for alice, bob, and charlie
function generate_accounts() {
  const accounts = ['alice', 'bob', 'charlie'];

  for (const name of accounts) {
    exe(`${stellar} keys generate ${name} --network testnet --overwrite`);
    exe(`${stellar} keys show ${name} > ${name}.txt`);
  }
}


function filenameNoExtension(filename) {
  return path.basename(filename, path.extname(filename));
}

function deploy(wasm) {

  let contractId = path.join(dirname, '.stellar', 'contract-ids', filenameNoExtension(wasm) + '.txt');

  exe(`(${stellar} contract deploy --wasm ${wasm} --ignore-checks --source-account alice --network testnet) > ${contractId}`);
}

function deploy_all() {
  const contractsDir = path.join(dirname, '.stellar', 'contract-ids');
  mkdirSync(contractsDir, { recursive: true });

  let wasmFiles = readdirSync(`${dirname}`).filter(file => file.endsWith('.wasm'));
  console.log(dirname);
  
  let rust_wasm = path.join('rust','target','wasm32v1-none', 'release-with-logs', 'hello_world.wasm');

  // add rust wasm file to the list of wasm files
  wasmFiles.push(rust_wasm);

  wasmFiles.forEach(wasmFile => {
    deploy(path.join(dirname, wasmFile));
  });
}

function add_testnet() {

  exe(`${stellar} network add \
    --global testnet \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015"`);
}

add_testnet();
generate_accounts();
deploy_all();
