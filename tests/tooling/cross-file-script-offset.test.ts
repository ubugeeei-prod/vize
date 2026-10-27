import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

type Message = {
  ruleId: string;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  [key: string]: unknown;
};
type Packet = { file: string; messages: Message[]; [key: string]: unknown };
type Point = { line: number; column: number };
type Span = { start: Point; end: Point };
type Anchor = { before: Span; after: Span };
type Block = { start: number; content: string; setup: boolean };
type Input = { name: string; app: string; child: string; templateControl: boolean };

const root = path.resolve(fileURLToPath(new URL("../../", import.meta.url)));
const issue = JSON.parse(
  fs.readFileSync(
    new URL("./fixtures/cross-file-script-offset/issue.json", import.meta.url),
    "utf8",
  ),
) as { number: number; html_url: string; body: string };
const call = 'provide("theme", "dark")';
const bytes = (source: string) => Buffer.byteLength(source);
const hash = (source: string | Buffer) => createHash("sha256").update(source).digest("hex");

function point(source: string, offset: number): Point {
  const prefix = Buffer.from(source)
    .subarray(0, Math.min(offset, bytes(source)))
    .toString("utf8");
  const lines = prefix.split("\n");
  // Match the CLI's Unicode scalar columns, including UTF-8 authored prefixes.
  return { line: lines.length, column: Array.from(lines.at(-1) ?? "").length + 1 };
}
function span(source: string, start: number, end: number): Span {
  return { start: point(source, start), end: point(source, end) };
}
function location(message: Message): Span {
  return {
    start: { line: message.line, column: message.column },
    end: { line: message.endLine, column: message.endColumn },
  };
}
function anchors(source: string, width = bytes(call)): Anchor[] {
  const blocks: Block[] = [...source.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)].map(
    (match) => ({
      start: bytes(source.slice(0, match.index + match[0].indexOf(">") + 1)),
      content: match[2],
      setup: /\bsetup\b/.test(match[1]),
    }),
  );
  const plain = blocks.find((block) => !block.setup);
  const setup = blocks.find((block) => block.setup);
  const first = plain ?? setup;
  assert(first);
  const result: Anchor[] = [];
  for (const block of blocks) {
    const at = block.content.indexOf(call);
    if (at < 0) continue;
    assert.equal(block.content.indexOf(call, at + call.length), -1);
    const local = bytes(block.content.slice(0, at));
    const virtual = block.setup && plain ? bytes(plain.content) + 1 + local : local;
    result.push({
      before: span(source, first.start + virtual, first.start + virtual + width),
      after: span(source, block.start + local, block.start + local + width),
    });
  }
  return result;
}
function inputs(): Input[] {
  assert.equal(issue.number, 10913);
  assert.equal(issue.html_url, "https://github.com/ubugeeei-prod/vize/issues/10913");
  const fence = String.fromCharCode(96).repeat(3);
  const originals = [...issue.body.matchAll(new RegExp(fence + "vue\\n([\\s\\S]*?)" + fence, "g"))];
  assert.equal(originals.length, 2);
  const result: Input[] = [
    {
      name: "original-issue",
      app: originals[0][1],
      child: originals[1][1],
      templateControl: false,
    },
  ];
  for (const [name, order, generic, crlf, utf8] of [
    ["same-provide-both-blocks", "plain-first", false, false, false],
    ["generic-tag", "plain-first", true, false, false],
    ["crlf-utf8", "plain-first", true, true, true],
    ["setup-before-plain", "setup-first", true, false, true],
    ["setup-only", "setup-only", false, false, false],
    ["plain-only", "plain-only", false, false, false],
  ] as const) {
    const caption = utf8 ? "日本語 café" : "settings";
    const pad = utf8 ? " ".repeat(60) : "";
    const plainBody =
      '\nimport { provide } from "vue";\nimport Child from "./Child.vue";\n' +
      'export const label = "' +
      caption +
      '";\n' +
      (utf8 ? 'const plainCafé = "日本"; ' + pad : "") +
      call +
      ";\n";
    const setupBody =
      '\nimport { provide } from "vue";\nimport Child from "./Child.vue";\n' +
      (utf8 ? 'const setupCafé = "日本"; ' + pad : "") +
      call +
      ";\n";
    const plain = '<script lang="ts">' + plainBody + "</script>\n";
    const setup =
      '<script setup lang="ts"' + (generic ? ' generic="T"' : "") + ">" + setupBody + "</script>\n";
    let app =
      (order === "setup-only"
        ? setup
        : order === "plain-only"
          ? plain
          : order === "setup-first"
            ? setup + plain
            : plain + setup) + '<template>\n  <div  id="shared"><Child /></div>\n</template>\n';
    let child = originals[1][1].replace("<p>{{ theme }}</p>", '<p  id="shared">{{ theme }}</p>');
    if (crlf) {
      app = app.replaceAll("\n", "\r\n");
      child = child.replaceAll("\n", "\r\n");
    }
    result.push({ name, app, child, templateControl: true });
  }
  return result;
}
function scriptProvide(message: Message): boolean {
  return (
    message.ruleId === "cross-file" &&
    /vize:croquis\/cf\/(?:non-reactive-provide|provide-without-symbol|unused-provide):/.test(
      message.message,
    )
  );
}
// Preserve the diagnostic's existing range law: the reactivity finding has no
// end offset and the CLI renders one byte; key/unused findings cover the call.
function diagnosticAnchors(source: string, message: Message): Anchor[] {
  const kind = message.message.match(
    /vize:croquis\/cf\/(non-reactive-provide|provide-without-symbol|unused-provide):/,
  )?.[1];
  assert(kind, "the script-span oracle must classify this exact finding");
  return anchors(source, kind === "non-reactive-provide" ? 1 : bytes(call));
}
function canonical(packets: Packet[]): Packet[] {
  return structuredClone(packets)
    .map((packet) => ({
      ...packet,
      messages: packet.messages.sort((a, b) =>
        JSON.stringify(a).localeCompare(JSON.stringify(b), "en"),
      ),
    }))
    .sort((a, b) => a.file.localeCompare(b.file, "en"));
}
function appPacket(packets: Packet[]): Packet {
  const packet = packets.find((entry) => entry.file.replaceAll("\\", "/").endsWith("src/App.vue"));
  assert(packet);
  return packet;
}
function expectedFromBefore(packets: Packet[], input: Input): Packet[] {
  const expected = structuredClone(packets);
  for (const message of appPacket(expected).messages.filter(scriptProvide)) {
    const anchor = diagnosticAnchors(input.app, message).find(
      (entry) => JSON.stringify(entry.before) === JSON.stringify(location(message)),
    );
    assert(
      anchor,
      input.name +
        ": every complete legacy script span has an authored owner: " +
        JSON.stringify(message),
    );
    message.line = anchor.after.start.line;
    message.column = anchor.after.start.column;
    message.endLine = anchor.after.end.line;
    message.endColumn = anchor.after.end.column;
  }
  return expected;
}

test("cross-file split-script coordinates preserve complete public packets and authored controls", () => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, identity.binaryPath + ".differential-build.json"), "utf8"),
  );
  validateBuildReceipt(receipt, identity);
  const candidate = path.join(root, identity.binaryPath);
  const baseline = process.env.VIZE_CROSS_FILE_OFFSET_BEFORE_CLI;
  if (process.env.VIZE_CROSS_FILE_OFFSET_REQUIRE_BEFORE === "1") assert(baseline);
  const output =
    process.env.VIZE_CROSS_FILE_OFFSET_PROOF ??
    path.join(root, "target/differential/cross-file-script-offset.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const evidence: {
    schema: string;
    issue: number;
    producer: unknown;
    baselineVersion?: unknown;
    cases: unknown[];
  } = {
    schema: "vize.cross-file.script-offset-packets.v1",
    issue: 10913,
    producer: { identity, receipt },
    cases: [],
  };
  const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-script-offset-"));
  const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
  delete env.FORCE_COLOR;
  function invoke(cli: string, args: string[]) {
    const run = spawnSync(cli, args, {
      cwd: workspace,
      env,
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    return {
      command: cli,
      cwd: workspace,
      args,
      status: run.status,
      signal: run.signal,
      error: run.error?.message ?? null,
      stdout: run.stdout ?? "",
      stderr: run.stderr ?? "",
    };
  }
  function validate(run: ReturnType<typeof invoke>) {
    assert.equal(run.error, null, JSON.stringify(run));
    assert.equal(run.signal, null, JSON.stringify(run));
    assert([0, 1].includes(run.status ?? -1), JSON.stringify(run));
  }
  const args = (format: string) => [
    "lint",
    "--no-config",
    "--cross-file",
    "--format",
    format,
    "--help-level",
    "none",
    "--locale",
    "en",
    "src/App.vue",
    "src/Child.vue",
  ];
  try {
    if (baseline) {
      evidence.baselineVersion = invoke(baseline, ["--version"]);
      persist();
      const version = evidence.baselineVersion as ReturnType<typeof invoke>;
      validate(version);
      assert.equal(version.status, 0);
      assert.match(version.stdout, /\b0\.601\.0\b/);
    }
    fs.mkdirSync(path.join(workspace, "src"));
    for (const input of inputs()) {
      fs.writeFileSync(path.join(workspace, "src/App.vue"), input.app);
      fs.writeFileSync(path.join(workspace, "src/Child.vue"), input.child);
      const entry = {
        input,
        sourceHashes: { app: hash(input.app), child: hash(input.child) },
        before: baseline ? invoke(baseline, args("json")) : null,
        after: invoke(candidate, args("json")),
        beforePlain: baseline ? invoke(baseline, args("plain")) : null,
        afterPlain: invoke(candidate, args("plain")),
        beforeDoctor: baseline
          ? invoke(baseline, ["doctor", "src", "--format", "json", "--exit-zero"])
          : null,
        afterDoctor: invoke(candidate, ["doctor", "src", "--format", "json", "--exit-zero"]),
      };
      evidence.cases.push(entry);
      persist();
      validate(entry.after);
      validate(entry.afterPlain);
      validate(entry.afterDoctor);
      const after = JSON.parse(entry.after.stdout) as Packet[];
      for (const message of appPacket(after).messages.filter(scriptProvide)) {
        assert(
          diagnosticAnchors(input.app, message).some(
            (anchor) => JSON.stringify(anchor.after) === JSON.stringify(location(message)),
          ),
          input.name + ": every script finding retains its exact authored range law",
        );
      }
      const provides = appPacket(after).messages.filter(
        (message) =>
          message.ruleId === "cross-file" &&
          message.message.includes("vize:croquis/cf/provide-without-symbol:"),
      );
      const expected = anchors(input.app)
        .map((anchor) => anchor.after)
        .sort((a, b) => a.start.line - b.start.line || a.start.column - b.start.column);
      assert.deepEqual(provides.map(location), expected, input.name + ": exact full call spans");
      if (input.name === "original-issue") {
        assert.deepEqual(expected, [
          { start: { line: 9, column: 1 }, end: { line: 9, column: 25 } },
        ]);
        assert.match(entry.afterPlain.stdout, /src\/App\.vue:9:1 warning cross-file/);
      }
      if (input.templateControl) {
        const duplicates = after
          .flatMap((packet) => packet.messages)
          .filter((message) => message.message.includes("vize:croquis/cf/duplicate-id:"));
        assert.equal(duplicates.length, 1, input.name);
        assert(
          after
            .flatMap((packet) => packet.messages)
            .some((message) => message.ruleId === "vue/no-multi-spaces"),
        );
      }
      if (entry.before && entry.beforePlain && entry.beforeDoctor) {
        validate(entry.before);
        validate(entry.beforePlain);
        validate(entry.beforeDoctor);
        const before = JSON.parse(entry.before.stdout) as Packet[];
        assert.deepEqual(
          canonical(after),
          canonical(expectedFromBefore(before, input)),
          input.name + ": every field except the bounded script coordinates stays identical",
        );
        assert.deepEqual(
          JSON.parse(entry.afterDoctor.stdout),
          JSON.parse(entry.beforeDoctor.stdout),
          input.name + ": the complete already-correct doctor packet stays identical",
        );
        if (input.name === "original-issue") {
          assert.notDeepEqual(canonical(after), canonical(before));
          assert.match(entry.beforePlain.stdout, /src\/App\.vue:7:1 warning cross-file/);
        }
      }
      assert.equal(hash(fs.readFileSync(path.join(workspace, "src/App.vue"))), hash(input.app));
      assert.equal(hash(fs.readFileSync(path.join(workspace, "src/Child.vue"))), hash(input.child));
    }
    assert.equal(evidence.cases.length, 7);
    console.log(
      "cross-file script-offset whole packets: " + evidence.cases.length + " controls passed",
    );
  } finally {
    persist();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
