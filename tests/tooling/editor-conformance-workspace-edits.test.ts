import assert from "node:assert/strict";
import { test } from "node:test";

import {
  evaluate,
  type Entry,
  type Scenario,
  WORKSPACE,
} from "../editor-conformance/conformance.ts";

const root = "/work/space";
const document = "src/Scenario.vue";
const uri = `file://${root}/${document}`;
const normalizedUri = `${WORKSPACE}/${document}`;
const range = { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } };
const edit = { range, newText: "b" };
const expectedAction = {
  title: "Replace text",
  kind: "quickfix",
  edit: { changes: { [normalizedUri]: [edit] } },
};
const scenario: Scenario = {
  suite: "workspace-edit-shape",
  document,
  languageId: "vue",
  initializationOptions: {},
  steps: [
    { id: "open", label: "open", kind: "open", expect: "a" },
    {
      id: "code-action",
      label: "code action",
      kind: "request",
      method: "textDocument/codeAction",
      match: { range },
      expect: [expectedAction],
      diagnosticsFromRequest: true,
    },
  ],
};

function transcript(documentChanges: boolean, version = 0, newText = "b"): Entry[] {
  const responseEdit = documentChanges
    ? {
        documentChanges: [{ textDocument: { uri, version }, edits: [{ ...edit, newText }] }],
      }
    : { changes: { [uri]: [{ ...edit, newText }] } };
  return [
    {
      session: 1,
      t: 0,
      dir: "c2s",
      msg: {
        id: 1,
        method: "initialize",
        params: { capabilities: { workspace: { workspaceEdit: { documentChanges } } } },
      },
    },
    {
      session: 1,
      t: 1,
      dir: "c2s",
      msg: {
        method: "textDocument/didOpen",
        params: { textDocument: { uri, languageId: "vue", version: 0, text: "a" } },
      },
    },
    {
      session: 1,
      t: 2,
      dir: "c2s",
      msg: {
        id: 2,
        method: "textDocument/codeAction",
        params: { textDocument: { uri }, range, context: { diagnostics: [] } },
      },
    },
    {
      session: 1,
      t: 3,
      dir: "s2c",
      msg: { id: 2, result: [{ ...expectedAction, edit: responseEdit }] },
    },
  ];
}

function failedSteps(entries: Entry[]): string[] {
  return evaluate(scenario, entries, [root])
    .steps.filter((step) => !step.pass)
    .map((step) => step.id);
}

test("the judge requires the exact WorkspaceEdit shape negotiated by the client", () => {
  assert.deepEqual(failedSteps(transcript(true)), []);
  assert.deepEqual(failedSteps(transcript(true, 1)), ["code-action"]);
  assert.deepEqual(failedSteps(transcript(true, 0, "wrong")), ["code-action"]);
  assert.deepEqual(failedSteps(transcript(false)), []);

  const unadvertisedVersioned = transcript(false);
  unadvertisedVersioned[3].msg!.result[0].edit = transcript(true)[3].msg!.result[0].edit;
  assert.deepEqual(failedSteps(unadvertisedVersioned), ["code-action"]);
});
