/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { ExtensionContext, Uri } from 'vscode';
import { LanguageClientOptions } from 'vscode-languageclient';

import { LanguageClient } from 'vscode-languageclient/browser';


// this method is called when vs code is activated
export function activate(context: ExtensionContext) {

	console.log('lsp-web-extension-sample activated!');

	/* 
	 * all except the code to create the language client in not browser specific
	 * and could be shared with a regular (Node) extension
	 */
	//const documentSelector = [{ language: 'plaintext' }];

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ language: 'solidity', scheme: 'file' },
			{ language: 'solidity', scheme: 'untitled' },
		],
	};

	const client = createWorkerLanguageClient(context, clientOptions);

	const disposable = client.start();
	//context.subscriptions.push(disposable);


}

function createWorkerLanguageClient(context: ExtensionContext, clientOptions: LanguageClientOptions) {
	// Create a worker. The worker main file implements the language server.
	const serverMain = Uri.joinPath(context.extensionUri, 'dist', 'web', 'language_server.js');
	const worker = new Worker(serverMain.toString(true));

	// create the language server client to communicate with the server running in the worker
	return new LanguageClient('solang', 'SOLANG', clientOptions, worker);
}