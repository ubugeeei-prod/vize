import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  array,
  integer,
  record,
  string,
  write,
  type Expected,
  type RecordValue,
  type Runtime,
} from "./model.ts";
import { PublicRefSession } from "./lsp.ts";
import type { Project } from "./fixtures.ts";
function owned(packet: RecordValue, project: Project, expected: Expected[]) {
  const params = record(packet.params),
    uri = string(params.uri);
  const file = expected[0]?.file ?? path.basename(new URL(uri).pathname);
  const diagnostics = array(params.diagnostics);
  assert.equal(diagnostics.length, expected.length);
  for (const [index, value] of diagnostics.entries()) {
    const diagnostic = record(value),
      start = record(record(diagnostic.range).start),
      end = record(record(diagnostic.range).end);
    const row = expected[index];
    assert.ok(row);
    assert.equal(uri, pathToFileURL(path.join(project.directory, row.file)).href);
    assert.deepEqual(
      {
        file: row.file,
        line: integer(start.line) + 1,
        column: integer(start.character) + 1,
        code: Number(diagnostic.code),
      },
      row,
    );
    assert.equal(integer(diagnostic.severity), 1);
    assert.ok(string(diagnostic.message).length > 0);
    assert.ok(integer(end.line) >= integer(start.line));
    assert.ok(integer(end.character) >= 0);
  }
  return { file, packet };
}
function hoverText(value: unknown): string {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(hoverText).join("\n");
  if (value && typeof value === "object") {
    const r = record(value);
    if (typeof r.value === "string") return r.value;
    if (r.contents !== undefined) return hoverText(r.contents);
  }
  return "";
}
export async function lspAdmission(
  runtime: Runtime,
  project: Project,
  states: readonly { name: string; files: Record<string, string>; expected: Expected[] }[],
  rows: Expected[],
  hovers: boolean,
) {
  const output = path.join(runtime.output, path.basename(project.directory), "lsp");
  const session = new PublicRefSession(runtime, project.directory, output);
  const publications: RecordValue[] = [];
  const fileUri = (file: string) => pathToFileURL(path.join(project.directory, file)).href;
  let primary: unknown;
  try {
    await session.initialize();
    for (const [index, state] of states.entries()) {
      const version = index + 1;
      for (const row of rows) {
        const text = state.files[row.file];
        assert.ok(typeof text === "string");
        fs.writeFileSync(path.join(project.directory, row.file), text);
        if (index === 0)
          session.notify("textDocument/didOpen", {
            textDocument: { uri: fileUri(row.file), languageId: "vue", version, text },
          });
        else
          session.notify("textDocument/didChange", {
            textDocument: { uri: fileUri(row.file), version },
            contentChanges: [{ text }],
          });
      }
      const packets = await Promise.all(
        rows.map((row) => session.diagnostics(fileUri(row.file), version)),
      );
      for (const [i, packet] of packets.entries()) {
        const row = rows[i];
        assert.ok(row);
        owned(
          packet,
          project,
          state.expected.filter((wanted) => wanted.file === row.file),
        );
        publications.push(packet);
      }
      write(path.join(output, state.name + "-whole-packets.txt"), packets);
      if (index === 0 && hovers) {
        const probes = [
          ["TypedDom-1.vue", "button.value?.click", "button", "HTMLButtonElement"],
          ["InferredDom-1.vue", "button.value?.click", "button", "HTMLButtonElement"],
          ["InferredComponent-1.vue", "child.value?.focus", "focus", "focused"],
          ["GenericComponent-1.vue", "generic.value?.value.extra", "extra", "boolean"],
        ];
        const replies: unknown[] = [];
        for (const probe of probes) {
          const [file, anchor, token, wanted] = probe;
          assert.ok(file && anchor && token && wanted);
          const text = state.files[file];
          assert.ok(text);
          const offset = text.indexOf(anchor) + anchor.indexOf(token);
          assert.ok(text.indexOf(anchor) >= 0 && anchor.indexOf(token) >= 0);
          const prefix = text.slice(0, offset).split(/\r\n|[\n\r\u2028\u2029]/u),
            tail = prefix.at(-1);
          assert.ok(tail !== undefined);
          const reply = await session.request("textDocument/hover", {
            textDocument: { uri: fileUri(file) },
            position: { line: prefix.length - 1, character: tail.length },
          });
          const rendered = hoverText(reply);
          assert.ok(rendered.includes(wanted));
          assert.doesNotMatch(rendered, /\b(?:any|unknown|never)\b|\$\$|__VLS/u);
          replies.push({ file, anchor, token, wanted, reply });
        }
        assert.equal(replies.length, 4);
        write(path.join(output, "typed-hover-whole-packets.txt"), replies);
      }
    }
  } catch (error) {
    primary = error;
  }
  try {
    await session.shutdown();
  } catch (error) {
    throw primary === undefined
      ? error
      : new AggregateError([primary, error], "public LSP admission and shutdown failed");
  }
  if (primary !== undefined) throw primary;
  // Audit every publication, including repeated or late envelopes, against its
  // source version; a first clean publication cannot hide a later real error.
  for (const packet of session.messages)
    if (packet.method === "textDocument/publishDiagnostics") {
      const params = record(packet.params),
        uri = string(params.uri);
      if (params.version === undefined || params.version === null) {
        const file = path.relative(project.directory, fileURLToPath(uri));
        assert.ok(Object.hasOwn(project.files, file), "unowned unversioned diagnostic URI");
        assert.deepEqual(
          array(params.diagnostics),
          [],
          "unversioned publications cannot conceal a real error",
        );
        continue;
      }
      const version = integer(params.version),
        state = states[version - 1];
      assert.ok(state);
      const row = rows.find((row) => fileUri(row.file) === uri);
      assert.ok(row, "unowned diagnostic URI");
      owned(
        packet,
        project,
        state.expected.filter((wanted) => wanted.file === row.file),
      );
    }
  assert.equal(publications.length, states.length * rows.length);
  return {
    documents: rows.length,
    versions: states.length,
    awaitedPublications: publications.length,
    allEnvelopes: session.messages.length,
    typedHovers: hovers ? 4 : 0,
  };
}
