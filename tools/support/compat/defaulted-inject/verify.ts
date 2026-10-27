import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../../tests/differential/build-receipt.ts";

type Message = {
  ruleId: string;
  message: string;
  severity: number;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
};
type Packet = { file: string; messages: Message[]; errorCount: number; warningCount: number };
type Input = {
  name: string;
  files: Record<string, string>;
  call: string;
  removed: number;
  required: number;
  retainedSeverity?: number;
};
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const output = process.env.VIZE_DEFAULTED_INJECT_PROOF;
const baseline = process.env.VIZE_DEFAULTED_INJECT_BEFORE_CLI;
assert(output && baseline, "proof output and original public before CLI are required");
const identity = expectedBuildIdentity(root);
const receipt = JSON.parse(
  fs.readFileSync(path.join(root, identity.binaryPath + ".differential-build.json"), "utf8"),
);
validateBuildReceipt(receipt, identity);
const candidate = path.join(root, identity.binaryPath);
const issue = JSON.parse(
  fs.readFileSync(new URL("./issue-10918.json.txt", import.meta.url), "utf8"),
) as { number: number; body: string };
assert.equal(issue.number, 10918);
const fence = String.fromCharCode(96).repeat(3);
const vue = [...issue.body.matchAll(new RegExp(fence + "vue\\n([\\s\\S]*?)" + fence, "g"))];
const ts = [...issue.body.matchAll(new RegExp(fence + "ts\\n([\\s\\S]*?)" + fence, "g"))];
assert.equal(vue.length, 3);
assert.equal(ts.length, 1);
const firstVue = vue[0]?.[1],
  secondVue = vue[1]?.[1],
  thirdVue = vue[2]?.[1],
  firstTs = ts[0]?.[1];
assert(firstVue && secondVue && thirdVue && firstTs);
const originalFiles = {
  "src/components/keys.ts": firstTs,
  "src/components/MyList.vue": firstVue,
  "src/components/MyDivider.vue": secondVue,
  "src/App.vue": thirdVue,
};
const originalCall = "inject(InListKey, false)";
const cases: Input[] = [];
for (const [name, call, removed, required, provider, outer] of [
  ["original-issue", originalCall, 1, 0, true, true],
  ["required-partial", "inject(InListKey)", 0, 1, true, true],
  ["undefined-default", "inject(InListKey, undefined)", 1, 0, true, true],
  ["null-default", "inject(InListKey, null)", 1, 0, true, true],
  ["factory-default", "inject(InListKey, () => false, true)", 1, 0, true, true],
  ["function-value-default", "inject(InListKey, () => false, false)", 1, 0, true, true],
  ["default-without-provider", originalCall, 1, 0, false, true],
  ["required-without-provider", "inject(InListKey)", 0, 1, false, true],
  ["all-branches-provided", originalCall, 0, 0, true, false],
] as const) {
  const files = structuredClone(originalFiles);
  assert.equal(files["src/components/MyDivider.vue"].split(originalCall).length, 2);
  files["src/components/MyDivider.vue"] = files["src/components/MyDivider.vue"].replace(
    originalCall,
    call,
  );
  if (!provider)
    files["src/components/MyList.vue"] = files["src/components/MyList.vue"].replace(
      "provide(InListKey, true);",
      "",
    );
  if (!outer)
    files["src/App.vue"] = files["src/App.vue"].replace("  <MyDivider />\n  <MyDivider />\n", "");
  cases.push({ name, files, call, removed, required });
}
for (const [name, call, prelude, removed, retainedSeverity] of [
  ["empty-literal-spread", "inject(InListKey, ...[])", "", 0, 1],
  ["empty-tuple-spread", "inject(InListKey, ...fallback)", "const fallback: [] = [];", 0, 1],
  [
    "unknown-tuple-spread",
    "inject(InListKey, ...fallback)",
    "declare const fallback: [] | [boolean];",
    0,
    1,
  ],
  [
    "undefined-before-flags",
    "inject(InListKey, undefined, ...flags)",
    "const flags: [] = [];",
    1,
    undefined,
  ],
  [
    "false-before-flags",
    "inject(InListKey, false, ...flags)",
    "const flags: [] = [];",
    1,
    undefined,
  ],
] as const) {
  const files = structuredClone(originalFiles);
  const divider = "src/components/MyDivider.vue";
  assert.equal(files[divider].split(originalCall).length, 2);
  assert.equal(files[divider].split("const inList =").length, 2);
  files[divider] = files[divider]
    .replace(originalCall, call)
    .replace("const inList =", prelude + "\nconst inList =");
  cases.push({
    name,
    files,
    call,
    removed,
    required: retainedSeverity === undefined ? 0 : 1,
    retainedSeverity,
  });
}
const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-defaulted-inject-"));
const hash = (source: string | Buffer) => createHash("sha256").update(source).digest("hex");
const evidence: {
  schema: string;
  issue: number;
  source: unknown;
  publicBefore: unknown;
  cases: unknown[];
} = {
  schema: "vize.optional-inject.whole-packets.v1",
  issue: 10918,
  source: { identity, receipt, binarySha256: hash(fs.readFileSync(candidate)) },
  publicBefore: null,
  cases: [],
};
const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
function invoke(cli: string, args: string[]) {
  const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
  delete env.FORCE_COLOR;
  const result = spawnSync(cli, args, {
    cwd: workspace,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  return {
    command: cli,
    args,
    cwd: workspace,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}
function valid(packet: ReturnType<typeof invoke>) {
  assert.equal(packet.error, null, JSON.stringify(packet));
  assert.equal(packet.signal, null, JSON.stringify(packet));
  assert([0, 1].includes(packet.status ?? -1), JSON.stringify(packet));
}
const tick = String.fromCharCode(96);
const bodies = [
  "**Conditionally Defaulted Inject**: " +
    tick +
    "inject(InListKey)" +
    tick +
    " falls back to its default value in 1 of 2 ancestor branches.",
  "**Unmatched Inject Default**: " +
    tick +
    "inject(InListKey)" +
    tick +
    " falls back to its default value because no ancestor provides this key.",
];
function optional(message: Message) {
  return (
    message.ruleId === "cross-file" &&
    message.message.includes("vize:croquis/cf/unmatched-inject:") &&
    bodies.some((body) => message.message.endsWith(body))
  );
}
try {
  evidence.publicBefore = invoke(baseline, ["--version"]);
  persist();
  const version = evidence.publicBefore as ReturnType<typeof invoke>;
  valid(version);
  assert.equal(version.status, 0);
  assert.match(version.stdout, /\b0\.602\.0\b/);
  for (const input of cases) {
    for (const [filename, source] of Object.entries(input.files)) {
      fs.mkdirSync(path.dirname(path.join(workspace, filename)), { recursive: true });
      fs.writeFileSync(path.join(workspace, filename), source);
    }
    const args = [
      "lint",
      "--no-config",
      "--cross-file",
      "--format",
      "json",
      "--help-level",
      "none",
      "--locale",
      "en",
      "src/**/*.vue",
    ];
    const before = invoke(baseline, args),
      after = invoke(candidate, args);
    const beforeDoctor = invoke(baseline, ["doctor", "src", "--format", "json", "--exit-zero"]);
    const afterDoctor = invoke(candidate, ["doctor", "src", "--format", "json", "--exit-zero"]);
    const entry = {
      input,
      hashes: Object.fromEntries(
        Object.entries(input.files).map(([name, value]) => [name, hash(value)]),
      ),
      before,
      after,
      beforeDoctor,
      afterDoctor,
      doctorScope:
        "Complete raw captured; unchanged controls exact; native Rust asserts graph and summary invariants.",
    };
    evidence.cases.push(entry);
    persist();
    for (const packet of [before, after, beforeDoctor, afterDoctor]) valid(packet);
    const original = JSON.parse(before.stdout) as Packet[],
      actual = JSON.parse(after.stdout) as Packet[];
    const expected = structuredClone(original);
    let removed = 0;
    for (const packet of expected) {
      const targets = input.retainedSeverity === undefined ? packet.messages.filter(optional) : [];
      if (targets.length > 0) {
        assert.equal(
          path.relative(workspace, path.resolve(workspace, packet.file)).replaceAll("\\", "/"),
          "src/components/MyDivider.vue",
        );
        const source = input.files["src/components/MyDivider.vue"];
        assert(source);
        const at = source.indexOf(input.call);
        assert(at >= 0);
        const prefix = source.slice(0, at).split("\n");
        const last = prefix.at(-1);
        assert(last !== undefined);
        const line = prefix.length,
          column = Array.from(last).length + 1;
        for (const target of targets) {
          assert.equal(target.severity, 1);
          assert.deepEqual(
            [target.line, target.column, target.endLine, target.endColumn],
            [line, column, line, column + input.call.length],
          );
        }
      }
      removed += targets.length;
      packet.messages = packet.messages.filter(
        (message) => input.retainedSeverity !== undefined || !optional(message),
      );
      packet.warningCount -= targets.length;
    }
    assert.equal(removed, input.removed, input.name + ": original whole finding admission");
    assert.deepEqual(
      actual,
      expected,
      input.name + ": complete output changes only by the owned optional finding",
    );
    const required = actual
      .flatMap((packet) => packet.messages)
      .filter(
        (message) =>
          message.ruleId === "cross-file" &&
          message.message.includes("vize:croquis/cf/unmatched-inject:"),
      );
    assert.equal(
      required.length,
      input.required,
      input.name + ": required injections retain errors",
    );
    for (const message of required) assert.equal(message.severity, input.retainedSeverity ?? 2);
    if (input.removed === 0) {
      assert.deepEqual(
        JSON.parse(afterDoctor.stdout),
        JSON.parse(beforeDoctor.stdout),
        input.name + ": unchanged complete doctor control",
      );
      assert.equal(after.status, before.status);
    }
    for (const [filename, source] of Object.entries(input.files))
      assert.equal(hash(fs.readFileSync(path.join(workspace, filename))), hash(source));
  }
  assert.equal(evidence.cases.length, 14);
  console.log(
    JSON.stringify({
      qualified: true,
      fullLintPacketPairs: 14,
      rawDoctorPairs: 14,
      role: "source-candidate-against-public-0.602",
      source: identity,
    }),
  );
} finally {
  persist();
  fs.rmSync(workspace, { recursive: true, force: true });
}
