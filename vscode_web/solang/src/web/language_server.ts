/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { createConnection, BrowserMessageReader, BrowserMessageWriter, Message } from 'vscode-languageserver/browser';

import { Color, ColorInformation, Range, InitializeParams, InitializeResult, ServerCapabilities, TextDocuments, ColorPresentation, TextEdit, TextDocumentIdentifier, WindowFeature, } from 'vscode-languageserver';
import { TextDocument } from 'vscode-languageserver-textdocument';
import init, { greet, start_server } from './pkg';
import wasmData from './pkg/solang_bg.wasm';

console.log('running server lsp-web-extension-sample');

/* browser specific setup code */

const messageReader = new BrowserMessageReader(self);
const messageWriter = new BrowserMessageWriter(self);

const connection = createConnection(messageReader, messageWriter);


connection.listen();

// on any message from the client, log it to the console


connection.onInitialize((params: InitializeParams): InitializeResult => {

    // send mock request to server
    let request = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": params,
        "id": 1
    };
    let response = "";
    init(wasmData).then(() => {
        let response = start_server(JSON.stringify(request));
        connection.console.log("response from server : " + JSON.stringify(response));
        let res_json = parseResponse(response);
        connection.console.log("res_json : " + JSON.stringify(res_json));

        let capabilities = res_json.result.capabilities;

        connection.console.log("capablitities : " + JSON.stringify(capabilities));


        const result: InitializeResult = { capabilities };
        return result;
    });

    return {
        capabilities: {
            textDocumentSync: {
                openClose: true,
                change: 1,
            },
            colorProvider: true,
            workspace: {
                workspaceFolders: {
                    supported: true,
                },
            },
            hoverProvider: true,
        },
    };



});



connection.onRequest((method, params, token) => {

    connection.console.log("method:: " + JSON.stringify(method));
    connection.console.log("params:: " + JSON.stringify(params));
    connection.console.log("token:: " + JSON.stringify(token));
    connection.console.log("=====================================");


    let request_json = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    };

    connection.console.log("request_json input to server " + JSON.stringify(request_json));

    // make new json with the content length

    init(wasmData).then(() => {
        let response = start_server(JSON.stringify(request_json));
        connection.console.log("response from server : " + JSON.stringify(response));
    });





    const capabilities: ServerCapabilities = {
        textDocumentSync: {
            openClose: true,
            change: 1,
        },
        colorProvider: true,
        workspace: {
            workspaceFolders: {
                supported: true,
            },
        },
        hoverProvider: true,
    };


    const result: InitializeResult = { capabilities };
    return result;
});

function parseResponse(response: string) {
    // Split the response into header and body
    const splitResponse = response.split('\r\n\r\n');

    // The second part is the body (JSON)
    const body = splitResponse[1];

    // Parse the body as JSON
    const json = JSON.parse(body);

    return json;
}