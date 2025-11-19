const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
    console.log('JCL extension activated');

    // Get configuration
    const config = vscode.workspace.getConfiguration('jcl');
    const lspEnabled = config.get('lsp.enabled', true);

    if (lspEnabled) {
        startLanguageServer(context, config);
    }

    // Register format command
    const formatCommand = vscode.commands.registerCommand('jcl.format', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'jcl') {
            return;
        }
        await vscode.commands.executeCommand('editor.action.formatDocument');
    });

    // Register lint command
    const lintCommand = vscode.commands.registerCommand('jcl.lint', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'jcl') {
            return;
        }
        // Trigger diagnostics refresh
        if (client) {
            await client.sendRequest('textDocument/diagnostic', {
                textDocument: { uri: editor.document.uri.toString() }
            });
        }
    });

    context.subscriptions.push(formatCommand, lintCommand);

    // Format on save if enabled
    vscode.workspace.onDidSaveTextDocument(async (document) => {
        if (document.languageId === 'jcl' && config.get('format.enabled', true)) {
            await vscode.commands.executeCommand('jcl.format');
        }
    });
}

function startLanguageServer(context, config) {
    const lspPath = config.get('lsp.path', 'jcl-lsp');

    // Server options
    const serverOptions = {
        command: lspPath,
        args: [],
        transport: TransportKind.stdio
    };

    // Client options
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'jcl' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.jcl')
        }
    };

    // Create and start the language client
    client = new LanguageClient(
        'jcl-lsp',
        'JCL Language Server',
        serverOptions,
        clientOptions
    );

    client.start().catch(err => {
        vscode.window.showErrorMessage(
            `Failed to start JCL Language Server: ${err.message}. ` +
            `Make sure jcl-lsp is installed and in your PATH.`
        );
    });

    context.subscriptions.push(client);
}

function deactivate() {
    if (client) {
        return client.stop();
    }
}

module.exports = {
    activate,
    deactivate
};
