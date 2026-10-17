import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { vueConfigFixtureSource } from "./vue-config-fixture.ts";

// The complete published formatter-indent-width recipe, including its output.
export const CONFIG_PROBE_INPUT = "<template><p>hello</p></template>";
export const CONFIG_PROBE_OUTPUT = "<template>\n   <p>hello</p>\n</template>\n";
const nativePolicy = '{"formatter":{"tabWidth":3}}\n';
export const RETIRED_FORMATTER_ERROR =
  "Invalid formatter configuration: failed to parse vize.config.json: vize.config.* is no longer supported; move Vue settings to vue: in vite.config.* and TypeScript settings to tsconfig.json\n";
const modes = ["legacy-native-json", "canonical-vue"] as const;
type Mode = (typeof modes)[number];
type Bytes = { base64: string; sha256: string };
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const bytes = (value: string | Buffer): Bytes => {
  const buffer = Buffer.from(value);
  return { base64: buffer.toString("base64"), sha256: digest(buffer) };
};
function decoded(value: Bytes): Buffer {
  const buffer = Buffer.from(value.base64, "base64");
  assert.equal(buffer.toString("base64"), value.base64, "lossless raw probe bytes required");
  assert.equal(digest(buffer), value.sha256, "raw probe digest changed");
  return buffer;
}
const configName = (mode: Mode) =>
  mode === "legacy-native-json" ? "vize.config.json" : "vite.config.mjs";
const configSource = (mode: Mode) =>
  mode === "legacy-native-json" ? nativePolicy : vueConfigFixtureSource(nativePolicy);
export type ConfigProbe = {
  mode: Mode;
  argv: string[];
  cwd: string;
  config: Bytes & { name: string };
  input: Bytes;
  output: Bytes;
  stdout: Bytes;
  stderr: Bytes;
  status: number | null;
  signal: string | null;
  error: string | null;
};
export type BenchmarkConfigContract = {
  schema: "vize.benchmark.actual-config-contract-v1";
  scope: "separate CLI qualification before any measured or priming LSP process";
  binary: { path: string; sha256: string };
  mode: Mode;
  probes: ConfigProbe[];
};

function policySucceeded(probe: ConfigProbe): boolean {
  return (
    probe.error === null &&
    probe.signal === null &&
    probe.status === 0 &&
    decoded(probe.output).equals(Buffer.from(CONFIG_PROBE_OUTPUT))
  );
}

/** The mode follows observed complete policy behavior, never a version guess. */
export function validateBenchmarkConfigContract(contract: BenchmarkConfigContract): void {
  assert.equal(contract.schema, "vize.benchmark.actual-config-contract-v1");
  assert.equal(
    contract.scope,
    "separate CLI qualification before any measured or priming LSP process",
  );
  assert.ok(path.isAbsolute(contract.binary.path));
  assert.match(contract.binary.sha256, /^[a-f0-9]{64}$/);
  assert.deepEqual(
    contract.probes.map((probe) => probe.mode),
    modes,
  );
  for (const probe of contract.probes) {
    assert.ok(path.isAbsolute(probe.cwd));
    assert.deepEqual(probe.argv, [
      "fmt",
      "--config",
      configName(probe.mode),
      "--write",
      "Probe.vue",
    ]);
    assert.equal(probe.config.name, configName(probe.mode));
    assert.deepEqual(probe.config, {
      name: configName(probe.mode),
      ...bytes(configSource(probe.mode)),
    });
    assert.deepEqual(probe.input, bytes(CONFIG_PROBE_INPUT));
    for (const raw of [probe.output, probe.stdout, probe.stderr]) decoded(raw);
  }
  const [legacy, canonical] = contract.probes;
  if (policySucceeded(legacy)) {
    assert.equal(
      contract.mode,
      "legacy-native-json",
      "accepted original policy keeps the historical carrier, including when both forms work",
    );
    return;
  }
  assert.equal(legacy.error, null);
  assert.equal(legacy.signal, null);
  assert.equal(legacy.status, 2, "unknown legacy-config failure cannot select a new contract");
  assert.deepEqual(legacy.stdout, bytes(""));
  assert.deepEqual(
    legacy.stderr,
    bytes(RETIRED_FORMATTER_ERROR),
    "only the exact dedicated-config retirement permits migration",
  );
  assert.deepEqual(
    legacy.output,
    bytes(CONFIG_PROBE_INPUT),
    "retired configuration must be refused before writing",
  );
  assert.ok(
    policySucceeded(canonical),
    "the canonical carrier must execute the full independently authored formatter policy",
  );
  assert.equal(contract.mode, "canonical-vue");
}

export function assertBenchmarkContractBinary(
  contract: BenchmarkConfigContract,
  binary: string,
): void {
  validateBenchmarkConfigContract(contract);
  assert.deepEqual(
    contract.binary,
    { path: fs.realpathSync(binary), sha256: digest(fs.readFileSync(binary)) },
    "config qualification belongs to this exact executable",
  );
}

/** Invoke before constructing an LSP session, capture, sampler, or priming sweep. */
export function qualifyBenchmarkConfig(
  binary: string,
  retainedPath?: string,
): BenchmarkConfigContract {
  const identity = { path: fs.realpathSync(binary), sha256: digest(fs.readFileSync(binary)) };
  fs.accessSync(identity.path, fs.constants.X_OK);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-contract-"));
  const probes: ConfigProbe[] = [];
  const retain = (record: unknown) => {
    if (!retainedPath) return;
    fs.mkdirSync(path.dirname(retainedPath), { recursive: true });
    fs.writeFileSync(retainedPath, `${JSON.stringify(record, null, 2)}\n`);
  };
  try {
    for (const mode of modes) {
      const cwd = path.join(directory, mode);
      fs.mkdirSync(cwd);
      fs.writeFileSync(path.join(cwd, "package.json"), '{"private":true,"type":"module"}\n');
      fs.writeFileSync(path.join(cwd, "Probe.vue"), CONFIG_PROBE_INPUT);
      const name = configName(mode);
      const source = configSource(mode);
      fs.writeFileSync(path.join(cwd, name), source);
      const argv = ["fmt", "--config", name, "--write", "Probe.vue"];
      const result = spawnSync(identity.path, argv, {
        cwd,
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      assert.equal(
        digest(fs.readFileSync(identity.path)),
        identity.sha256,
        "executable changed during config qualification",
      );
      assert.equal(
        fs.readFileSync(path.join(cwd, name), "utf8"),
        source,
        "probe policy bytes stay fixed",
      );
      probes.push({
        mode,
        argv,
        cwd,
        config: { name, ...bytes(source) },
        input: bytes(CONFIG_PROBE_INPUT),
        output: bytes(fs.readFileSync(path.join(cwd, "Probe.vue"))),
        stdout: bytes(result.stdout ?? Buffer.alloc(0)),
        stderr: bytes(result.stderr ?? Buffer.alloc(0)),
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
      });
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
  const contract: BenchmarkConfigContract = {
    schema: "vize.benchmark.actual-config-contract-v1",
    scope: "separate CLI qualification before any measured or priming LSP process",
    binary: identity,
    mode: policySucceeded(probes[0]) ? "legacy-native-json" : "canonical-vue",
    probes,
  };
  retain(contract);
  assertBenchmarkContractBinary(contract, binary);
  return contract;
}

/** Preserve the original config document exactly; derive only the active carrier. */
export function writeBenchmarkConfig(
  workspace: string,
  original: string | Buffer,
  contract: BenchmarkConfigContract,
) {
  validateBenchmarkConfigContract(contract);
  const native = Buffer.from(original);
  const current =
    contract.mode === "legacy-native-json" ? native : Buffer.from(vueConfigFixtureSource(native));
  const name = configName(contract.mode);
  if (contract.mode === "canonical-vue")
    fs.writeFileSync(path.join(workspace, "vize.config.json.fixture"), native, { flag: "wx" });
  fs.writeFileSync(path.join(workspace, name), current, { flag: "wx" });
  assert.deepEqual(
    fs.readFileSync(path.join(workspace, name)),
    current,
    "actual active policy bytes",
  );
  return {
    contract,
    original: { name: "vize.config.json", ...bytes(native) },
    ...(contract.mode === "canonical-vue" ? { evidencePath: "vize.config.json.fixture" } : {}),
    current: { name, ...bytes(current) },
  };
}
