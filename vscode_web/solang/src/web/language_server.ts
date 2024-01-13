/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { createConnection, BrowserMessageReader, BrowserMessageWriter, Message, WriteableStreamMessageWriter, ReadableStreamMessageReader, createMessageConnection, } from 'vscode-languageserver/browser';

import { Color, ColorInformation, Range, InitializeParams, InitializeResult, ServerCapabilities, TextDocuments, ColorPresentation, TextEdit, TextDocumentIdentifier, WindowFeature, } from 'vscode-languageserver';
import { TextDocument } from 'vscode-languageserver-textdocument';
import init, { greet, initSync, start_listener, ServerConfig } from './pkg';
import wasmData from './pkg/solang_bg.wasm';

console.log('running server lsp-web-extension-sample');

/* browser specific setup code */


const messageReader = new BrowserMessageReader(self);
const messageWriter = new BrowserMessageWriter(self);

//const otherMessageReader = new ReadableStreamMessageReader(self as any as ReadableStream<Message>);

//let sesa = new self.ReadableByteStreamController();

const connection = createConnection(messageReader, messageWriter);

connection.listen();

connection.console.log("hello world");


async function main() {







    await init(wasmData).then((module) => {
        connection.console.log("wasm loaded");

    }).catch((err) => {
        connection.console.log("wasm error");
        connection.console.log(err);
    }).finally(() => {
        connection.console.log("wasm finally");
    });

    //connection.console.log("after wasm");

    //start_listener(messageWriter);

    connection.onInitialize((params: InitializeParams): InitializeResult => {
        connection.console.log("onInitialize");


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

    connection.onRequest((method: string, params: any) => {
        connection.console.log("onRequest");



        //let sesa = greet("hello");
        //connection.console.log(sesa);

        //return null;
    });




}




main()


function parseResponse(response: string) {
    // Split the response into header and body
    const splitResponse = response.split('\r\n\r\n');

    // The second part is the body (JSON)
    const body = splitResponse[1];

    // Parse the body as JSON
    const json = JSON.parse(body);

    return json;
}