// One cold editor session as a user sees it: open an SFC, wait for the
// extension's activation (its client has started) and for the first published
// Vize diagnostic. Times are milliseconds after the open request.
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

exports.run = async function run() {
  const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!root) throw new Error("startup latency needs a workspace folder");
  const uri = vscode.Uri.file(path.join(root, process.env.VIZE_BENCH_FILE));
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  if (!extension || extension.isActive) throw new Error("Vize must start inactive");
  const timeoutMs = Number(process.env.VIZE_BENCH_TIMEOUT_MS || 120_000);
  const marks = {};
  let start;
  let resolveDiagnostic;
  const diagnosed = new Promise((resolve) => {
    resolveDiagnostic = resolve;
  });
  const subscription = vscode.languages.onDidChangeDiagnostics(() => {
    if (marks.firstDiagnostic !== undefined || start === undefined) return;
    const published = vscode.languages
      .getDiagnostics(uri)
      .filter((diagnostic) => String(diagnostic.source ?? "").startsWith("vize"));
    if (published.length === 0) return;
    marks.firstDiagnostic = performance.now() - start;
    marks.diagnostics = published.length;
    resolveDiagnostic();
  });
  let timer;
  let epoch;
  try {
    epoch = Date.now();
    start = performance.now();
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    marks.shown = performance.now() - start;
    await extension.activate();
    marks.activated = performance.now() - start;
    await Promise.race([
      diagnosed,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error("no Vize diagnostic was published")), timeoutMs);
      }),
    ]);
  } finally {
    clearTimeout(timer);
    subscription.dispose();
  }
  const rounded = Object.fromEntries(
    Object.entries(marks).map(([key, value]) => [key, Math.round(value * 10) / 10]),
  );
  fs.writeFileSync(process.env.VIZE_BENCH_RESULT, `${JSON.stringify({ ...rounded, epoch })}\n`);
};
