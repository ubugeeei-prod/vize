import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";
import {
  regularMember,
  sourceCliCheckout,
  sharedOracleAuthority,
} from "../../tools/support/compat/github/source-cli-identity.ts";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI for contract tests, mirroring the LSP
 * smoke-test launcher: a prebuilt binary wins, otherwise fall back to cargo so
 * fresh checkouts still work.
 */
function resolveVizeCommand(): { command: string; prefix: string[] } {
  const candidates = [
    path.join(root, "target/ci/vize"),
    path.join(root, "target/release/vize"),
    path.join(root, "target/debug/vize"),
    "vize",
  ];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: root, encoding: "utf8" });
    if (probe.status === 0) {
      return { command: candidate, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

const VIZE = resolveVizeCommand();

type CheckResult = { status: number | null; stdout: string; stderr: string };

function runCheck(args: string[], cwd: string): CheckResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", ...args], {
    cwd,
    encoding: "utf8",
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

// The workspace must live outside the repository tree: `vize check` with no
// explicit inputs walks up looking for a tsconfig.json, and a repo-local temp
// dir would inherit the workspace tsconfig and start type-checking the repo.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-check-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("vize check --help documents the input contract and key flags", () => {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  const help = result.stdout;
  for (const flag of ["--format", "--no-config", "--tsconfig", "--max-warnings", "--quiet"]) {
    assert.match(help, new RegExp(flag.replace(/-/g, "\\-")), `help should document ${flag}`);
  }
  assert.match(help, /PATTERNS/, "help should describe the positional PATTERNS argument");
});

test("vize check exits 1 and reports SFC parse errors before type checking", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Broken.vue"), "<template><div></div>", "utf8");
    const result = runCheck(["Broken.vue"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /SFC parse error/);
  });
});

test("vize check reports parse errors even with config loading disabled", () => {
  // The parse failure must short-circuit before the Corsa type-check stage, so
  // this assertion holds without a type-checker present.
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Bad.vue"), "<template><div>", "utf8");
    const result = runCheck(["Bad.vue", "--no-config"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /SFC parse error/);
  });
});

test("vize check exits 2 when an explicit config file is missing", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Ok.vue"), "<template><p /></template>\n", "utf8");
    const result = runCheck(["Ok.vue", "-c", "./does-not-exist.json"], dir);
    assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /config file not found/);
  });
});

test("vize check exits 0 when explicit inputs match no supported files", () => {
  withWorkspace((dir) => {
    const result = runCheck(["does-not-exist.vue"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(
      result.stderr,
      'No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: ["does-not-exist.vue"]\n',
    );
  });
});

test("vize check exits 0 in an empty project with no inputs", () => {
  withWorkspace((dir) => {
    const result = runCheck([], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(
      result.stderr,
      "No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: []\n",
    );
  });
});

const preparationInput = fs.readFileSync(
  new URL(
    "../_fixtures/differential/typechecker/check-parse-before-runtime-10732/cases.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(preparationInput).digest("hex"),
  "c24a743240082791caed1803ec2aa9153accba6330690723ef8d9dd2ed6fb4ae",
);
const preparationCases = JSON.parse(preparationInput.toString("utf8")) as {
  parseCases: Array<{ id: string; source: string; expected: CheckResult }>;
  validCase: { source: string; expected: CheckResult };
  configErrorCase: { argument: string; expected: CheckResult };
};

const preparationPackets: Array<{ argv: string[]; inputSha256: string; result: CheckResult }> = [];
let preparationProducer:
  | {
      sourceRevision: string;
      producerReceiptSha256: string;
      useReceiptSha256: string | null;
      binary: ReturnType<typeof regularMember>;
    }
  | undefined;

function requirePreparationProducer() {
  if (preparationProducer) return preparationProducer;
  const producerReceiptSha256 = process.env.VIZE_SOURCE_CLI_RECEIPT_SHA256;
  const useReceiptSha256 = process.env.VIZE_SOURCE_CLI_USE_RECEIPT_SHA256;
  assert.ok(producerReceiptSha256, "the original source CLI producer receipt is required");
  const checkout = sourceCliCheckout(root);
  const producer = sharedOracleAuthority(
    root,
    path.join(root, "target/ci/examples/cli_build_result_oracle"),
  );
  assert.ok(producer, "the original same-attempt or verified reused source producer is required");
  assert.equal(VIZE.command, path.join(root, "target/ci/vize"));
  const binary = regularMember(root, "target/ci/vize");
  assert.deepEqual(
    binary,
    producer.members[0],
    "the actual source CLI image must match its producer",
  );
  preparationProducer = {
    sourceRevision: checkout.sourceRevision,
    producerReceiptSha256,
    useReceiptSha256: useReceiptSha256 ?? null,
    binary,
  };
  return preparationProducer;
}

after(() => {
  assert.equal(preparationPackets.length, 9, "all actual no-SDK packet witnesses are required");
  const producer = requirePreparationProducer();
  assert.deepEqual(
    regularMember(root, "target/ci/vize"),
    producer.binary,
    "the source CLI image changed during packet observations",
  );
  assert.equal(sourceCliCheckout(root).sourceRevision, producer.sourceRevision);
  console.log(
    JSON.stringify({
      schema: "vize.check-preparation.packets",
      version: 1,
      fixtureSha256: createHash("sha256").update(preparationInput).digest("hex"),
      producer,
      packets: preparationPackets,
    }),
  );
});

/** Run the real built CLI with no inherited SDK paths, package manager or home. */
function runCheckWithoutRuntime(args: string[], cwd: string): CheckResult {
  assert.equal(VIZE.prefix.length, 0, "isolated runtime contracts require the built source CLI");
  assert(path.isAbsolute(VIZE.command), "isolated runtime contracts require an absolute CLI path");
  requirePreparationProducer();
  const inputSha256 = createHash("sha256")
    .update(fs.readFileSync(path.join(cwd, args[0])))
    .digest("hex");
  const home = path.join(cwd, "isolated-home");
  const bin = path.join(home, "bin");
  fs.mkdirSync(bin, { recursive: true });
  const result = spawnSync(VIZE.command, ["check", "--quiet", ...args], {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
    env: {
      HOME: home,
      USERPROFILE: home,
      XDG_CACHE_HOME: path.join(home, "cache"),
      LOCALAPPDATA: path.join(home, "cache"),
      APPDATA: path.join(home, "config"),
      PATH: bin,
      NO_COLOR: "1",
      ...(process.platform === "win32" ? { SystemRoot: process.env.SystemRoot } : {}),
    },
  });
  if (result.error) throw result.error;
  const packet = { status: result.status, stdout: result.stdout, stderr: result.stderr };
  preparationPackets.push({ argv: ["check", "--quiet", ...args], inputSha256, result: packet });
  return packet;
}

for (const extra of [[], ["--no-config"]]) {
  void test(`vize check preserves complete SFC parse failures without an SDK (${extra.length ? "no-config" : "config discovery"})`, () => {
    withWorkspace((dir) => {
      for (const entry of preparationCases.parseCases) {
        fs.writeFileSync(path.join(dir, "Broken.vue"), entry.source);
        assert.deepEqual(
          runCheckWithoutRuntime(["Broken.vue", ...extra], dir),
          entry.expected,
          entry.id,
        );
      }
    });
  });
}

void test("vize check keeps configuration precedence and complete valid-project missing-runtime help", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Broken.vue"), "<template><div>");
    assert.deepEqual(
      runCheckWithoutRuntime(
        ["Broken.vue", "--config", preparationCases.configErrorCase.argument],
        dir,
      ),
      preparationCases.configErrorCase.expected,
    );
    fs.writeFileSync(path.join(dir, "Valid.vue"), preparationCases.validCase.source);
    for (const extra of [[], ["--no-config"]]) {
      assert.deepEqual(
        runCheckWithoutRuntime(["Valid.vue", ...extra], dir),
        preparationCases.validCase.expected,
      );
    }
  });
});
