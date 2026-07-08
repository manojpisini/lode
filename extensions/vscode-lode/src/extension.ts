import * as vscode from 'vscode';
import * as path from 'path';
import * as cp from 'child_process';

let diagnosticCollection: vscode.DiagnosticCollection;
let statusBarItem: vscode.StatusBarItem;
let decorationType: vscode.TextEditorDecorationType;

interface Violation {
  file: string;
  line: number;
  column: number;
  message: string;
  severity: 'error' | 'warning' | 'info';
  rule: string;
}

export function activate(context: vscode.ExtensionContext): void {
  diagnosticCollection = vscode.languages.createDiagnosticCollection('lode');
  context.subscriptions.push(diagnosticCollection);

  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBarItem.command = 'lode.status';
  context.subscriptions.push(statusBarItem);

  decorationType = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(255, 100, 100, 0.12)',
    isWholeLine: true,
    overviewRulerColor: 'rgba(255, 100, 100, 0.6)',
    overviewRulerLane: vscode.OverviewRulerAspect.Right,
  });
  context.subscriptions.push(decorationType);

  registerCommands(context);
  registerEventHandlers(context);

  updateStatus();
  runDiagnostics();
}

function getBinaryPath(): string {
  const config = vscode.workspace.getConfiguration('lode');
  return config.get<string>('binaryPath', 'lode');
}

function getLodeConfigDir(): vscode.Uri | undefined {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) return undefined;
  return vscode.Uri.joinPath(folders[0].uri, '.lode');
}

function registerCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('lode.check', () => runLodeCheck()),
    vscode.commands.registerCommand('lode.scan', () => runLodeScan()),
    vscode.commands.registerCommand('lode.init', () => runLodeInit()),
    vscode.commands.registerCommand('lode.sync', () => runLodeSync()),
    vscode.commands.registerCommand('lode.status', () => showStatus()),
  );
}

function registerEventHandlers(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.uri.fsPath.includes('.lode')) {
        runDiagnostics();
      }
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('lode')) {
        updateStatus();
        runDiagnostics();
      }
    }),
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) updateDecorations(editor);
    }),
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document === vscode.window.activeTextEditor?.document) {
        updateDecorations(e.document);
      }
    }),
  );
}

function execLode(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const binary = getBinaryPath();
    const options: cp.ExecOptions = {
      cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
      maxBuffer: 10 * 1024 * 1024,
    };
    cp.exec(`${binary} ${args.join(' ')}`, options, (error, stdout, stderr) => {
      if (error && !stdout) {
        reject(new Error(stderr || error.message));
      } else {
        resolve(stdout || stderr);
      }
    });
  });
}

async function runLodeCheck(): Promise<void> {
  vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'LODE: Checking conventions...' }, async () => {
    try {
      const output = await execLode(['check', '--format', 'json']);
      parseAndShowViolations(output);
    } catch (err: any) {
      vscode.window.showErrorMessage(`LODE check failed: ${err.message}`);
    }
  });
}

async function runLodeScan(): Promise<void> {
  vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'LODE: Scanning secrets...' }, async () => {
    try {
      const output = await execLode(['scan']);
      const channel = vscode.window.createOutputChannel('LODE Scan');
      channel.clear();
      channel.appendLine(output);
      channel.show();
    } catch (err: any) {
      vscode.window.showErrorMessage(`LODE scan failed: ${err.message}`);
    }
  });
}

async function runLodeInit(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    vscode.window.showErrorMessage('No workspace folder open.');
    return;
  }
  const template = await vscode.window.showQuickPick(['default', 'minimal', 'strict'], {
    placeHolder: 'Select a LODE template (default: default)',
  });
  if (!template) return;
  vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'LODE: Initializing project...' }, async () => {
    try {
      const args = ['init', '--template', template];
      const output = await execLode(args);
      vscode.window.showInformationMessage('LODE initialized.');
      updateStatus();
      runDiagnostics();
    } catch (err: any) {
      vscode.window.showErrorMessage(`LODE init failed: ${err.message}`);
    }
  });
}

async function runLodeSync(): Promise<void> {
  vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'LODE: Syncing templates...' }, async () => {
    try {
      const output = await execLode(['sync']);
      vscode.window.showInformationMessage('LODE templates synced.');
    } catch (err: any) {
      vscode.window.showErrorMessage(`LODE sync failed: ${err.message}`);
    }
  });
}

async function showStatus(): Promise<void> {
  try {
    const output = await execLode(['status', '--format', 'json']);
    const data = JSON.parse(output);
    const items: vscode.QuickPickItem[] = [
      { label: 'LODE Project', description: data.project || 'unknown' },
      { label: 'Template', description: data.template || '—' },
      { label: 'Rules', description: String(data.rules_count ?? '?') },
      { label: 'Secrets scanned', description: String(data.secrets_scanned ?? '?') },
      { label: 'Last check', description: data.last_check || 'never' },
    ];
    vscode.window.showQuickPick(items, { placeHolder: 'LODE Status' });
  } catch {
    const configDir = getLodeConfigDir();
    if (configDir) {
      vscode.window.showQuickPick(
        [{ label: 'LODE project', description: configDir.fsPath }],
        { placeHolder: 'LODE: No status available' },
      );
    } else {
      vscode.window.showWarningMessage('No LODE project detected.');
    }
  }
}

async function updateStatus(): Promise<void> {
  statusBarItem.text = '$(check) LODE';
  try {
    const output = await execLode(['status', '--format', 'json']);
    const data = JSON.parse(output);
    if (data.violations > 0) {
      statusBarItem.text = `$(error) LODE: ${data.violations} violations`;
      statusBarItem.tooltip = `${data.violations} convention violations found`;
      statusBarItem.color = new vscode.ThemeColor('errorForeground');
    } else {
      statusBarItem.text = '$(check) LODE: OK';
      statusBarItem.tooltip = 'No violations';
      statusBarItem.color = new vscode.ThemeColor('foreground');
    }
  } catch {
    const configDir = getLodeConfigDir();
    if (configDir) {
      statusBarItem.text = '$(circuit-board) LODE';
      statusBarItem.tooltip = 'LODE project detected';
      statusBarItem.color = new vscode.ThemeColor('foreground');
    } else {
      statusBarItem.text = '$(circle-slash) LODE';
      statusBarItem.tooltip = 'No LODE project';
      statusBarItem.color = new vscode.ThemeColor('disabledForeground');
    }
  }
  statusBarItem.show();
}

async function runDiagnostics(): Promise<void> {
  const enabled = vscode.workspace.getConfiguration('lode').get<boolean>('enableDiagnostics', true);
  if (!enabled) return;
  diagnosticCollection.clear();

  const configDir = getLodeConfigDir();
  if (!configDir) return;

  try {
    const output = await execLode(['check', '--format', 'json']);
    parseAndSetDiagnostics(output);
  } catch {
    // diagnostics for config files only
  }
}

function parseAndSetDiagnostics(output: string): void {
  const uriMap = new Map<string, vscode.Diagnostic[]>();
  let violations: Violation[] = [];
  try {
    const parsed = JSON.parse(output);
    if (Array.isArray(parsed)) {
      violations = parsed;
    } else if (parsed.violations) {
      violations = parsed.violations;
    }
  } catch {
    return;
  }
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  for (const v of violations) {
    let filePath = v.file;
    if (workspaceRoot && !path.isAbsolute(filePath)) {
      filePath = path.join(workspaceRoot, filePath);
    }
    const uri = vscode.Uri.file(filePath);
    const severity = v.severity === 'error' ? vscode.DiagnosticSeverity.Error
      : v.severity === 'warning' ? vscode.DiagnosticSeverity.Warning
      : vscode.DiagnosticSeverity.Information;
    const diagnostic = new vscode.Diagnostic(
      new vscode.Range(v.line - 1, (v.column || 1) - 1, v.line - 1, 1000),
      v.message,
      severity,
    );
    diagnostic.source = 'lode';
    diagnostic.code = v.rule;
    const existing = uriMap.get(uri.fsPath) || [];
    existing.push(diagnostic);
    uriMap.set(uri.fsPath, existing);
  }
  for (const [fsPath, diags] of uriMap) {
    diagnosticCollection.set(vscode.Uri.file(fsPath), diags);
  }
}

function parseAndShowViolations(output: string): void {
  const channel = vscode.window.createOutputChannel('LODE Check');
  channel.clear();
  channel.appendLine('LODE Convention Check Results');
  channel.appendLine('='.repeat(40));
  channel.appendLine('');
  try {
    const parsed = JSON.parse(output);
    const violations: Violation[] = Array.isArray(parsed) ? parsed : parsed.violations || [];
    if (violations.length === 0) {
      channel.appendLine('No violations found.');
    } else {
      for (const v of violations) {
        channel.appendLine(`[${v.severity.toUpperCase()}] ${v.file}:${v.line}:${v.column || 1}`);
        channel.appendLine(`  Rule: ${v.rule}`);
        channel.appendLine(`  ${v.message}`);
        channel.appendLine('');
      }
      channel.appendLine(`Total: ${violations.length} violation(s)`);
    }
  } catch {
    channel.appendLine(output);
  }
  channel.show();
  parseAndSetDiagnostics(output);
}

function updateDecorations(documentOrEditor: vscode.TextDocument | vscode.TextEditor): void {
  const enabled = vscode.workspace.getConfiguration('lode').get<boolean>('enableDecorations', true);
  if (!enabled) return;

  const document = documentOrEditor instanceof vscode.TextEditor ? documentOrEditor.document : documentOrEditor;
  if (!document || document.uri.scheme !== 'file') return;

  const diags = diagnosticCollection.get(document.uri);
  if (!diags || diags.length === 0) {
    if (vscode.window.activeTextEditor?.document === document) {
      vscode.window.activeTextEditor.setDecorations(decorationType, []);
    }
    return;
  }
  const ranges = diags.map((d) => d.range);
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document === document) {
    editor.setDecorations(decorationType, ranges);
  }
}

export function deactivate(): void {
  diagnosticCollection?.dispose();
  statusBarItem?.dispose();
}
