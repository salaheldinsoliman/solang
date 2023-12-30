// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import init, { greet } from './pkg';
import wasmData from './pkg/solang_bg.wasm';
//import * as wasm from './pkg'


//import * as server from "./pkg/solang"
// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
const outputChannel = vscode.window.createOutputChannel("My Extension Logs");

export function activate(context: vscode.ExtensionContext) {




	init(wasmData).then(() => {
		outputChannel.appendLine("WASM initialized");
		let sesa = greet("sasa");
		outputChannel.appendLine(sesa);
	}
	);


	// Will be something like "file:///Users/billti/src/bqm/dist/hello_wasm_bg.wasm" running in VSCode.
	// Something like "http://localhost:3000/static/devextensions/dist/hello_wasm_bg.wasm" with npx @vscode/test-web

	outputChannel.appendLine("called activate");
	outputChannel.appendLine("anyyyyyyyy111");


	//outputChannel.appendLine(sesa);

	//outputChannel.appendLine("Hello World from solang!");
}

// This method is called when your extension is deactivated
export function deactivate() { }
