import * as vscode from 'vscode';
import * as path from 'path';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Aether');

    // Start LSP client
    startLanguageServer(context);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('aether.run', runCurrentFile),
        vscode.commands.registerCommand('aether.debug', debugCurrentFile),
        vscode.commands.registerCommand('aether.repl', openRepl),
        vscode.commands.registerCommand('aether.check', checkCurrentFile),
    );

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
        } else {
            statusBar.hide();
        }
    });

    if (vscode.window.activeTextEditor?.document.languageId === 'aether') {
        statusBar.show();
    }

    outputChannel.appendLine('Aether extension activated');
}

export function deactivate(): Thenable<void> | undefined {
    if (client) {
        return client.stop();
    }
    return undefined;
}

// ── LSP Client ──────────────────────────────────────────────────

function startLanguageServer(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('aether');
    const aetherPath = config.get<string>('path', 'aether');

    const serverOptions: ServerOptions = {
        command: aetherPath,
        args: ['lsp'],
        transport: TransportKind.stdio,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'aether' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ae'),
        },
        outputChannel,
    };

    client = new LanguageClient(
        'aether-lsp',
        'Aether Language Server',
        serverOptions,
        clientOptions,
    );

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
    const aetherPath = config.get<string>('path', 'aether');
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
    const aetherPath = config.get<string>('path', 'aether');

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
    const aetherPath = config.get<string>('path', 'aether');
    const filePath = editor.document.uri.fsPath;

    const cp = require('child_process');
    cp.exec(`${aetherPath} check "${filePath}"`, (err: any, stdout: string, stderr: string) => {
        if (err) {
            outputChannel.appendLine(stderr || stdout);
            outputChannel.show();
            vscode.window.showWarningMessage('Type errors found. See output for details.');
        } else {
            vscode.window.showInformationMessage('No type errors found!');
        }
    });
}
