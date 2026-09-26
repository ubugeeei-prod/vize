import assert from "node:assert/strict";
import {
  hoverToText,
  offsetToPosition,
} from "../../../../../tests/tooling/support/lsp/assertions.ts";
import { componentSource, documentUri } from "./lsp-resource-workspace.mjs";

export async function metadataCompletion(session, uri, source, type = "string") {
  const result = await session.request(
    "textDocument/completion",
    {
      textDocument: { uri },
      position: offsetToPosition(source, source.indexOf(":label=", source.indexOf("<template>"))),
      context: { triggerKind: 1 },
    },
    120_000,
  );
  const items = Array.isArray(result) ? result : result?.items;
  const prop = items?.find((item) => item.label === "label" || item.label === ":label");
  assert.ok(prop, `imported label prop completion missing for ${uri}`);
  assert.match(prop.detail, new RegExp(`prop: ${type} \\(required\\)`));
  return prop;
}

export async function controlledEdits(session, workspace) {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  const parent = componentSource(1);
  let child = componentSource(0);
  let version = 1;
  const proof = [];
  const change = (next) => {
    child = next;
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: ++version },
      contentChanges: [{ text: child }],
    });
  };
  const diagnostic = async (uri, documentVersion, code, pattern) => {
    const params = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) =>
        value.uri === uri &&
        value.version === documentVersion &&
        value.diagnostics.some((entry) => entry.code === code && pattern.test(entry.message)),
      180_000,
    );
    return params.diagnostics.find((entry) => entry.code === code && pattern.test(entry.message));
  };
  const start = performance.now();
  change(child.replace("checked: number = 1", 'checked: number = "resource-sentinel"'));
  proof.push({
    edit: "body type error",
    version,
    diagnostic: await diagnostic(
      childUri,
      version,
      2322,
      /Type 'string' is not assignable to type 'number'/,
    ),
  });
  await metadataCompletion(session, parentUri, parent);
  proof.push({ edit: "body change", prop_type: "string", elapsed_ms: performance.now() - start });

  change(child.replace('checked: number = "resource-sentinel"', "checked: number = 1"));
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) =>
      value.uri === childUri &&
      value.version === version &&
      !value.diagnostics.some((entry) => entry.code === 2322),
    180_000,
  );
  change(child.replace("label: string", "label: number"));
  await metadataCompletion(session, parentUri, parent, "number");
  proof.push({
    edit: "public prop",
    version,
    completion_type: "number",
    diagnostic: await diagnostic(parentUri, 1, 2322, /string.*number/),
  });

  change(
    child
      .replace("label: number", "label: string")
      .replace("save: [value: string]", "save: [value: number]"),
  );
  await metadataCompletion(session, parentUri, parent);
  const hover = await session.request(
    "textDocument/hover",
    {
      textDocument: { uri: parentUri },
      position: offsetToPosition(parent, parent.indexOf("\nPeer\n") + 2),
    },
    120_000,
  );
  assert.match(hoverToText(hover), /save: \[value: number\]/);
  proof.push({ edit: "public emit", version, hover: hoverToText(hover) });

  change(child.replace("publicState: string = 'ready'", "publicState: number = 1"));
  await metadataCompletion(session, parentUri, parent);
  proof.push({
    edit: "public expose",
    version,
    diagnostic: await diagnostic(parentUri, 1, 2339, /toUpperCase.*number/),
  });

  change(componentSource(0));
  await metadataCompletion(session, parentUri, parent);
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) =>
      value.uri === parentUri &&
      value.version === 1 &&
      !value.diagnostics.some((entry) => entry.code === 2339 || entry.code === 2322),
    180_000,
  );
  proof.push({ edit: "restore interface", version, parent_diagnostics_cleared: true });
  return proof;
}
