// Zed has no headless mode that runs language servers from a script, so this
// row is a *protocol replay*: the initialize parameters are transcribed from
// Zed 1.16.1's own client (`clients/zed-1.16.1.json` names the source), the
// launch command and initialization options follow the packaged Vize Zed
// extension (`editors/zed/src/lib.rs`: `vize` on PATH, `["lsp"]`, user
// `lsp.vize.initialization_options` replacing the recommended ones), and every
// request uses the parameter shapes Zed sends. The results table labels the
// row `replay`, never `editor`.
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  step,
  suiteRoot,
  settleDiagnostics,
  transcriptLength,
  type Driver,
  type DriverContext,
} from "../support/context.ts";
import { applyTextEdits, workspaceEditsFor, type TextEdit } from "../support/document.ts";
import { type Json, LspClient } from "../support/lsp-client.ts";
import { latestPublish, readTranscript } from "../support/transcript.ts";

const zed = JSON.parse(fs.readFileSync(path.join(suiteRoot, "clients", "zed-1.16.1.json"), "utf8"));

function which(binary: string, env: NodeJS.ProcessEnv): string {
  for (const dir of (env.PATH ?? "").split(path.delimiter)) {
    const candidate = path.join(dir, binary);
    if (fs.existsSync(candidate)) return candidate;
  }
  throw new Error(`${binary} is not on PATH`);
}

async function run(context: DriverContext): Promise<void> {
  const { scenario } = context;
  const rootUri = pathToFileURL(context.workspace).href;
  const uri = pathToFileURL(context.documentPath).href;
  const client = new LspClient(
    which("vize", context.env),
    ["lsp"],
    { cwd: context.workspace, env: context.env },
    (method, params: any) =>
      method === "workspace/configuration" ? params.items.map(() => null) : null,
  );
  let text = fs.readFileSync(context.documentPath, "utf8");
  let version = 0;
  const change = async (edits: TextEdit[]) => {
    // Zed syncs incrementally (the server asks for kind 2); one change per edit, last edit first.
    const ordered = [...edits].sort(
      (a, b) =>
        b.range.start.line - a.range.start.line ||
        b.range.start.character - a.range.start.character,
    );
    version += 1;
    client.notify("textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: ordered.map((edit) => ({ range: edit.range, text: edit.newText })) as Json,
    });
    text = applyTextEdits(text, edits);
  };
  const at = (position: Json) => ({ textDocument: { uri }, position });

  await client.request("initialize", {
    processId: process.pid,
    rootPath: context.workspace,
    rootUri,
    initializationOptions: scenario.initializationOptions as Json,
    capabilities: zed.capabilities,
    workspaceFolders: [{ uri: rootUri, name: path.basename(context.workspace) }],
    clientInfo: zed.clientInfo,
  });
  client.notify("initialized", {});

  let mark = transcriptLength(context);
  client.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version, text },
  });
  await settleDiagnostics(
    context,
    mark,
    step(scenario, "diagnostics-open", "diagnostics").expect.length,
  );

  const hover = step(scenario, "hover", "request").match as { position: Json };
  await client.request("textDocument/hover", at(hover.position));
  const completion = step(scenario, "completion", "request").match as { position: Json };
  await client.request("textDocument/completion", {
    ...at(completion.position),
    context: { triggerKind: 1 },
  });
  const definition = step(scenario, "definition", "request").match as { position: Json };
  await client.request("textDocument/definition", at(definition.position));

  const codeAction = step(scenario, "code-action", "request").match as { range: any };
  const published = latestPublish(
    readTranscript(context.transcript),
    context.documentKey,
    context.roots,
  );
  const overlapping = (published?.params.diagnostics ?? []).filter(
    (d: any) =>
      d.range.start.line <= codeAction.range.end.line &&
      d.range.end.line >= codeAction.range.start.line,
  );
  const actions = (await client.request("textDocument/codeAction", {
    textDocument: { uri },
    range: codeAction.range,
    context: { diagnostics: overlapping, triggerKind: 1 },
  })) as any[];
  const fix = step(scenario, "apply-quick-fix", "text").action as { title: string };
  mark = transcriptLength(context);
  await change(workspaceEditsFor(actions.find((action) => action.title === fix.title)?.edit, uri));
  await settleDiagnostics(
    context,
    mark,
    step(scenario, "diagnostics-quick-fix", "diagnostics").expect.length,
  );

  // Zed's LSP formatting options: tab size and soft tabs from the language
  // settings, plus its save-time whitespace settings (both default on).
  const formatting = (await client.request("textDocument/formatting", {
    textDocument: { uri },
    options: {
      tabSize: 2,
      insertSpaces: true,
      trimTrailingWhitespace: true,
      insertFinalNewline: true,
      trimFinalNewlines: true,
    },
  })) as TextEdit[];
  await change(formatting);

  const typed = step(scenario, "edit", "text").action as { replace: any; text: string };
  mark = transcriptLength(context);
  await change([{ range: typed.replace, newText: typed.text }]);
  await settleDiagnostics(
    context,
    mark,
    step(scenario, "diagnostics-edit", "diagnostics").expect.length,
  );

  const rename = step(scenario, "rename", "request").match as { position: Json; newName: string };
  await client.request("textDocument/prepareRename", at(rename.position));
  const edit = await client.request("textDocument/rename", {
    ...at(rename.position),
    newName: rename.newName,
  });
  await change(workspaceEditsFor(edit, uri));

  await client.request("shutdown", undefined);
  client.notify("exit", undefined);
  await client.exit();
}

export const driver: Driver = {
  client: "Zed",
  version: () => zed.clientInfo.version,
  mode: "replay",
  run,
};
