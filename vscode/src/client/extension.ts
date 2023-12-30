// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
// to run the vscode extension, press F5 or click the green arrow button in the debug panel
import * as vscode from 'vscode';
import * as rpc from 'vscode-jsonrpc';
import { promises as fs } from 'fs';
import { LanguageClient, LanguageClientOptions, ServerOptions, Executable } from 'vscode-languageclient';
import expandPathResolving from '../utils/expandPathResolving';
import init, { greet } from '../pkg/solang';
//import { Wasm } from '@vscode/wasm-wasi';
import { commands, ExtensionContext, Uri, window, workspace } from 'vscode';
import getServer from '../utils/getServer';

// this method is called when your extension is activated
// your extension is activated the very first time the command is executed
const outputChannel = vscode.window.createOutputChannel("My Extension Logs");







export async function load_wasm(context: vscode.ExtensionContext) {

  init().then(() => {
    greet("Solang wasm hellloooooo");
  }
  ).catch((err) => {
    console.log(err);
  });

}




export async function activate(context: vscode.ExtensionContext) {
  console.log('heeeeeeeeeeeeeeeeeeeeeeere');



  load_wasm(context);
  // Load the WASM module
  //const module = await wasm.createProcess();

  //greet("Solang wasm hellloooooo");
  await tryActivate(context).catch((err) => {
    void vscode.window.showErrorMessage(`Cannot activate solang: ${err.message}`);
    throw err;
  });
}

async function tryActivate(context: vscode.ExtensionContext) {
  await fs.mkdir(context.globalStoragePath, { recursive: true });

  const path = await bootstrapServer(context);
  await bootstrapExtension(context, path);
}

async function bootstrapExtension(context: vscode.ExtensionContext, serverpath: string) {
  const config = vscode.workspace.getConfiguration('solang');
  const target: string = config.get('target') || 'polkadot';

  // Use the console to output diagnostic information (console.log) and errors (console.error)
  // This line of code will only be executed once when your extension is activated
  console.log('Congratulations, your extension "solang" is now active!');

  const diagnosticCollection = vscode.languages.createDiagnosticCollection('solidity');

  context.subscriptions.push(diagnosticCollection);

  const connection = rpc.createMessageConnection(
    new rpc.StreamMessageReader(process.stdout),
    new rpc.StreamMessageWriter(process.stdin)
  );

  connection.listen();

  const sop: Executable = {
    command: expandPathResolving(serverpath),
    args: ['language-server', '--target', target],
  };

  const serverOptions: ServerOptions = sop;

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { language: 'solidity', scheme: 'file' },
      { language: 'solidity', scheme: 'untitled' },
    ],
  };

  const client = new LanguageClient('solidity', 'Solang Solidity Compiler', serverOptions, clientOptions).start();

  context.subscriptions.push(client);
}


async function bootstrapServer(context: vscode.ExtensionContext) {
  let path

  path = "solang";
  if (!path) {
    throw new Error('Solang Language Server is not available.');
  }

  outputChannel.appendLine("SOLANG: " + path);
  console.log('Using server binary at', path);

  return path;
}
