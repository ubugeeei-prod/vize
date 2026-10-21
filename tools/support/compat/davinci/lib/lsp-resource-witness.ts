import assert from "node:assert/strict";
import {
  hoverToText,
  offsetToPosition,
} from "../../../../../tests/tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../../../../tests/tooling/support/lsp/protocol.ts";
import type { LspSession } from "../../../../../tests/tooling/support/lsp/session.ts";
import { asString, at, isRecord } from "../../../json-value.ts";
import type { JsonRecord } from "../../../json-value.ts";
import { componentSource, documentUri } from "./lsp-resource-workspace.ts";

function isPublication(value: unknown): value is PublishDiagnosticsParams & object {
  return isRecord(value) && Array.isArray(value.diagnostics) && value.diagnostics.every(isRecord);
}

export async function metadataCompletion(
  session: LspSession,
  uri: string,
  source: string,
  type = "string",
): Promise<JsonRecord> {
  const result = await session.request(
    "textDocument/completion",
    {
      textDocument: { uri },
      position: offsetToPosition(source, source.indexOf(":label=", source.indexOf("<template>"))),
      context: { triggerKind: 1 },
    },
    120_000,
  );
  const items: unknown = Array.isArray(result) ? result : at(result, "items");
  const prop: unknown = Array.isArray(items)
    ? items.find(
        (item: unknown) => isRecord(item) && (item.label === "label" || item.label === ":label"),
      )
    : undefined;
  assert.ok(isRecord(prop), `imported label prop completion missing for ${uri}`);
  assert.match(asString(prop.detail, "detail"), new RegExp(`prop: ${type} \\(required\\)`));
  return prop;
}

export async function bodyEditChurn(
  session: LspSession,
  workspace: string,
  count = 32,
): Promise<{ edits: number; unchanged_prop_completion_replies: number; final_version: number }> {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  let version = 1;
  for (let edit = 0; edit < count; edit++) {
    const value = edit + 1 === count ? 1 : edit + 2;
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: ++version },
      contentChanges: [
        { text: componentSource(0).replace("checked: number = 1", `checked: number = ${value}`) },
      ],
    });
    await metadataCompletion(session, parentUri, componentSource(1));
  }
  return { edits: count, unchanged_prop_completion_replies: count, final_version: version };
}

export async function controlledEdits(
  session: LspSession,
  workspace: string,
  initialVersion = 1,
): Promise<JsonRecord[]> {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  const parent = componentSource(1);
  let child = componentSource(0);
  let version = initialVersion;
  let publications = 0;
  let afterEdit = 0;
  const publicationSequence = new WeakMap<object, number>();
  const sequenceOf = (value: object): number => publicationSequence.get(value) ?? 0;
  session.notificationObservers.push((method, params) => {
    if (method === "textDocument/publishDiagnostics" && isPublication(params)) {
      publicationSequence.set(params, ++publications);
    }
  });
  const proof: JsonRecord[] = [];
  const change = (next: string): void => {
    child = next;
    afterEdit = publications;
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: ++version },
      contentChanges: [{ text: child }],
    });
  };
  const diagnostic = async (
    uri: string,
    documentVersion: number,
    code: number,
    pattern: RegExp,
  ): Promise<PublishDiagnosticsParams["diagnostics"][number] | undefined> => {
    const params = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) =>
        isPublication(value) &&
        sequenceOf(value) > afterEdit &&
        value.uri === uri &&
        value.version === documentVersion &&
        value.diagnostics.some(
          (entry) => entry.code === code && pattern.test(String(entry.message)),
        ),
      180_000,
    );
    assert.ok(isPublication(params));
    return params.diagnostics.find(
      (entry) => entry.code === code && pattern.test(String(entry.message)),
    );
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
      isPublication(value) &&
      sequenceOf(value) > afterEdit &&
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
  const hoverResult = isRecord(hover) ? hover : null;
  assert.match(hoverToText(hoverResult), /save: \[value: number\]/);
  proof.push({ edit: "public emit", version, hover: hoverToText(hoverResult) });

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
      isPublication(value) &&
      sequenceOf(value) > afterEdit &&
      value.uri === parentUri &&
      value.version === 1 &&
      !value.diagnostics.some((entry) => entry.code === 2339 || entry.code === 2322),
    180_000,
  );
  proof.push({ edit: "restore interface", version, parent_diagnostics_cleared: true });
  return proof;
}
