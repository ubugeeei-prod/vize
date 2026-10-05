import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { hoverToText, offsetToPosition } from "../../tooling/support/lsp/assertions.ts";
import type { LspSession } from "../../tooling/support/lsp/session.ts";

export type RequestSpec = { name: string; method: string; params: unknown };
export type Packet = { name: string; method: string; result: unknown };

export function object(value: unknown): Record<string, unknown> {
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export const hoverText = (value: unknown): string => hoverToText(object(value));

export function requests(uri: string, source: string): RequestSpec[] {
  const position = (needle: string, inside: number, last = false) => {
    const offset = last ? source.lastIndexOf(needle) : source.indexOf(needle);
    assert.notEqual(offset, -1, needle);
    return offsetToPosition(source, offset + inside);
  };
  const at = (p: ReturnType<typeof offsetToPosition>) => ({ textDocument: { uri }, position: p });
  return [
    {
      name: "script hover open",
      method: "textDocument/hover",
      params: at(position("const open =", 8)),
    },
    {
      name: "template hover label",
      method: "textDocument/hover",
      params: at(position("{{ label }}", 5, true)),
    },
    {
      name: "definition useThing0",
      method: "textDocument/definition",
      params: at(position("useThing0(size)", 4)),
    },
    {
      name: "completion thing.",
      method: "textDocument/completion",
      params: at(position("thing.count.value", "thing.".length)),
    },
  ];
}

export function assertTypedPackets(packets: Packet[], workspace: string): void {
  assert.deepEqual(
    packets.map((packet) => packet.name),
    ["script hover open", "template hover label", "definition useThing0", "completion thing."],
  );
  for (const packet of packets) assert.notEqual(packet.result, null, packet.name);
  assert.match(hoverText(packets[0].result), /\b(?:boolean|number)\b/u);
  assert.match(hoverText(packets[1].result), /\bstring\b/u);
  const completion = packets[3].result;
  const items = Array.isArray(completion) ? completion : object(completion).items;
  assert.ok(Array.isArray(items));
  const labels = items.map((item: unknown) => object(item).label);
  for (const label of ["add", "count", "double", "items"]) assert.ok(labels.includes(label), label);
  const definition = packets[2].result;
  const locations = Array.isArray(definition) ? definition : [definition];
  assert.ok(locations.length > 0);
  for (const location of locations) {
    const raw = object(location);
    const uri = raw.uri ?? raw.targetUri;
    assert.equal(typeof uri, "string");
    assert.equal(fileURLToPath(uri as string), path.join(workspace, "src/lib/mod0.ts"));
    assert.ok(raw.range ?? raw.targetSelectionRange);
  }
}

/** One side owns a bijection of physical private storage roots, shared by every row. */
export class SessionRoots {
  readonly roots: string[] = [];
  readonly spellings = new Map<string, string>();

  visit(value: unknown): unknown {
    if (typeof value === "string") {
      for (const match of value.matchAll(
        /\/[^\s"<>]*?\/vize-canon\/editor\/sessions\/session-[^/\s"<>]+\//gu,
      )) {
        const spelling = match[0];
        if (!this.spellings.has(spelling)) {
          const root = fs.realpathSync(spelling);
          assert.equal(
            path.dirname(root),
            path.join(fs.realpathSync(os.tmpdir()), "vize-canon/editor/sessions"),
          );
          const stat = fs.statSync(root);
          assert.ok(stat.isDirectory());
          assert.equal(stat.mode & 0o777, 0o700);
          if (!this.roots.includes(root)) this.roots.push(root);
          this.spellings.set(spelling, `$EDITOR_SESSION_${this.roots.indexOf(root) + 1}/`);
        }
      }
      let result = value;
      for (const [spelling, token] of this.spellings) result = result.replaceAll(spelling, token);
      return result;
    }
    if (Array.isArray(value)) return value.map((item) => this.visit(item));
    if (value != null && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, this.visit(item)]),
      );
    }
    return value;
  }
}

export async function waitForVersion(
  session: LspSession,
  uri: string,
  version: number,
  start: number,
): Promise<void> {
  const markers = [
    `sent collected diagnostics for ${uri} version ${version}`,
    `finished initial type diagnostics for ${uri} version ${version}`,
  ];
  const deadline = Date.now() + 300_000;
  while (!markers.some((marker) => session.stderrText.slice(start).includes(marker))) {
    assert.ok(
      Date.now() < deadline,
      `complete current native phase missing: ${markers.join(" or ")}\n${session.stderrText}`,
    );
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

export async function waitForInitialTypes(session: LspSession, uri: string): Promise<void> {
  const marker = `finished initial type diagnostics for ${uri} version 1`;
  const deadline = Date.now() + 300_000;
  while (!session.stderrText.includes(marker)) {
    assert.ok(
      Date.now() < deadline,
      `initial native phase missing: ${marker}\n${session.stderrText}`,
    );
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  // Match the existing bounded Canon protocol's ten-second settling period.
  await new Promise((resolve) => setTimeout(resolve, 10_000));
}

export async function finishWire(repoRoot: string, previous: string[]) {
  const directory = path.join(repoRoot, "target/differential/lsp-sessions");
  const created = fs.readdirSync(directory).filter((name) => !previous.includes(name));
  assert.equal(created.length, 1, "each side owns exactly one actual server capture");
  const capture = path.join(directory, created[0]);
  const deadline = Date.now() + 5_000;
  let observation: Record<string, unknown>;
  while (true) {
    observation = JSON.parse(
      fs.readFileSync(path.join(capture, "observation.json"), "utf8"),
    ) as Record<string, unknown>;
    if (observation.state === "process-closed") break;
    assert.ok(Date.now() < deadline, "actual process-close observation did not arrive");
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  assert.deepEqual(observation.process, { exitStatus: 0, signal: null, error: null });
  const streams = object(observation.streams);
  for (const kind of ["client", "server", "stderr"]) {
    const stream = object(streams[kind]);
    assert.equal(stream.truncated, false);
    assert.equal(stream.capturedBytes, stream.observedBytes);
    if (kind !== "stderr") assert.equal(object(stream.framing).state, "complete");
  }
  return { capture, observation };
}
