import { createConnection, InitializeResult, DefinitionRequest, ProposedFeatures, InitializeParams, TextDocumentSyncKind } from 'vscode-languageserver';
import * as rpc from 'vscode-jsonrpc';
//import * as solang from 'solang';
import * as vscode from 'vscode';

import * as fs from "fs";
import * as path from "path";


declare const WebAssembly: any;


const connection = createConnection(
  new rpc.StreamMessageReader(process.stdin),
  new rpc.StreamMessageWriter(process.stdout)
);

connection.onInitialize(() => {
  //solang.trial();
  vscode.window.showInformationMessage('Hello World from solang!');
  console.log('initializing sesa kosomk');
  const result: InitializeResult = {
    capabilities: {},
  };
  return result;
});

connection.onInitialized(() => {
  connection.client.register(DefinitionRequest.type, undefined);
});

const notif = new rpc.NotificationType<string, void>('test notif');

connection.onNotification(notif, (param: string) => {
  console.log('notified\n');
  console.log(param);
});

connection.listen();
