"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
let outputChannel;
function activate(context) {
    outputChannel = vscode.window.createOutputChannel('Aether');
    // Start LSP client
    startLanguageServer(context);
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('aether.run', runCurrentFile), vscode.commands.registerCommand('aether.debug', debugCurrentFile), vscode.commands.registerCommand('aether.repl', openRepl), vscode.commands.registerCommand('aether.check', checkCurrentFile));
    // Status bar
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBar.text = '$(play) Aether';
    statusBar.tooltip = 'Run Aether file';
    statusBar.command = 'aether.run';
    context.subscriptions.push(statusBar);
    // Show status bar for .ae files
    vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor?.document.languageId === 'aether') {
            statusBar.show();
        }
        else {
            statusBar.hide();
        }
    });
    if (vscode.window.activeTextEditor?.document.languageId === 'aether') {
        statusBar.show();
    }
    outputChannel.appendLine('Aether extension activated');
}
function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}
// ── LSP Client ──────────────────────────────────────────────────
function startLanguageServer(context) {
    const config = vscode.workspace.getConfiguration('aether');
    const aetherPath = config.get('path', 'aether');
    const serverOptions = {
        command: aetherPath,
        args: ['lsp'],
        transport: node_1.TransportKind.stdio,
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'aether' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ae'),
        },
        outputChannel,
    };
    client = new node_1.LanguageClient('aether-lsp', 'Aether Language Server', serverOptions, clientOptions);
    client.start().then(() => {
        outputChannel.appendLine('Aether LSP server connected');
    }).catch((err) => {
        outputChannel.appendLine(`LSP failed to start: ${err}`);
        outputChannel.appendLine('IntelliSense will be limited. Install aether and ensure it\'s in PATH.');
    });
    context.subscriptions.push(client);
}
// ── Run Command ─────────────────────────────────────────────────
async function runCurrentFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aether') {
        vscode.window.showErrorMessage('No Aether file is open');
        return;
    }
    // Save first
    await editor.document.save();
    const config = vscode.workspace.getConfiguration('aether');
    const aetherPath = config.get('path', 'aether');
    const filePath = editor.document.uri.fsPath;
    const terminal = vscode.window.createTerminal({
        name: 'Aether Run',
        shellPath: aetherPath,
        shellArgs: ['run', filePath],
    });
    terminal.show();
}
// ── Debug Command ───────────────────────────────────────────────
async function debugCurrentFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aether') {
        vscode.window.showErrorMessage('No Aether file is open');
        return;
    }
    await editor.document.save();
    vscode.debug.startDebugging(undefined, {
        type: 'aether',
        request: 'launch',
        name: 'Debug Aether',
        program: editor.document.uri.fsPath,
    });
}
// ── REPL Command ────────────────────────────────────────────────
function openRepl() {
    const config = vscode.workspace.getConfiguration('aether');
    const aetherPath = config.get('path', 'aether');
    const terminal = vscode.window.createTerminal({
        name: 'Aether REPL',
        shellPath: aetherPath,
        shellArgs: ['repl'],
    });
    terminal.show();
}
// ── Check Command ───────────────────────────────────────────────
async function checkCurrentFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aether') {
        vscode.window.showErrorMessage('No Aether file is open');
        return;
    }
    await editor.document.save();
    const config = vscode.workspace.getConfiguration('aether');
    const aetherPath = config.get('path', 'aether');
    const filePath = editor.document.uri.fsPath;
    const cp = require('child_process');
    cp.exec(`${aetherPath} check "${filePath}"`, (err, stdout, stderr) => {
        if (err) {
            outputChannel.appendLine(stderr || stdout);
            outputChannel.show();
            vscode.window.showWarningMessage('Type errors found. See output for details.');
        }
        else {
            vscode.window.showInformationMessage('No type errors found!');
        }
    });
}
//# sourceMappingURL=extension.js.map