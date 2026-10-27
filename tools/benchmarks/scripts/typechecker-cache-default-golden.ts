/** Before/after default capture compatibility; this is untimed and uses public fixture bytes. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { parseArgs } from "node:util";
import { createColdScaleRuntime } from "./typechecker-cache-runtime.ts";

const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    before: { type: "string" },
    after: { type: "string" },
    out: { type: "string" },
  },
});
assert.ok(values.provider && values.before && values.after && values.out);
const out = path.resolve(values.out);
assert.ok(!fs.existsSync(out));
fs.mkdirSync(out, { recursive: true });
const project = path.join(out, "project");
fs.mkdirSync(path.join(project, "src"), { recursive: true });
// Physical public copies keep every library within the authored project for the legacy default capture contract.
fs.cpSync(path.join(values.provider, "node_modules"), path.join(project, "node_modules"), {
  recursive: true,
  dereference: true,
});
fs.writeFileSync(
  path.join(project, "tsconfig.json"),
  JSON.stringify(
    {
      compilerOptions: {
        strict: true,
        noEmit: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        skipLibCheck: true,
      },
      include: ["src/**/*.ts"],
    },
    null,
    2,
  ),
);
fs.writeFileSync(path.join(project, "src/main.ts"), "export const label: string = '日本😀';\n");
const backend = path.join(
  project,
  "node_modules",
  "@typescript",
  `typescript-${process.platform}-${process.arch}`,
  "lib",
  "tsc",
);
assert.ok(fs.existsSync(backend));
const results = [];
for (const [role, binary] of [
  ["before", values.before],
  ["after", values.after],
]) {
  const directory = path.join(out, role);
  fs.mkdirSync(directory);
  const capture = path.join(directory, "capture");
  const runtime = createColdScaleRuntime(directory);
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    ...runtime.env,
    VIZE_LEXICAL_DOLLAR_CAPTURE_DIR: capture,
    NO_COLOR: "1",
  };
  delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_METADATA_ONLY;
  try {
    const args = [
      "check",
      "--no-config",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      backend,
    ];
    const run = spawnSync(binary, args, {
      cwd: project,
      env,
      encoding: "utf8",
      timeout: 120000,
      maxBuffer: 64 * 1024 * 1024,
    });
    fs.writeFileSync(path.join(directory, "stdout.json"), run.stdout ?? "");
    fs.writeFileSync(path.join(directory, "stderr.txt"), run.stderr ?? "");
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    assert.equal(run.status, 0, run.stderr);
    const packet = JSON.parse(run.stdout);
    assert.equal(packet.errorCount, 0);
    assert.equal(packet.warningCount, 0);
    const names = fs
      .readdirSync(capture)
      .filter((name) => name.endsWith(".json"))
      .sort();
    assert.ok(names.length > 0);
    const normalized = names
      .map((name) => {
        const raw = JSON.parse(fs.readFileSync(path.join(capture, name), "utf8"));
        assert.equal(raw.completion, "REAPED");
        assert.equal(
          raw.metadataOnly,
          undefined,
          "the existing default must not gain metadata-mode fields",
        );
        assert.equal(raw.compiledSourceRevision, undefined);
        assert.ok(Array.isArray(raw.generatedFiles) && raw.generatedFiles.length > 0);
        assert.equal(typeof raw.projectRoot, "string");
        assert.equal(typeof raw.virtualRoot, "string");
        assert.equal(typeof raw.requestedConfigBytes, "string");
        assert.ok(Array.isArray(raw.actualArgs));
        assert.match(raw.nativeSha256, /^[a-f0-9]{64}$/u);
        assert.ok(
          raw.generatedFiles.some((file: { authoredSource?: string }) =>
            file.authoredSource?.includes("日本😀"),
          ),
          "the byte golden must contain the actual authored input",
        );
        const text = JSON.stringify(raw, null, 2)
          .replaceAll(raw.virtualRoot, "$VIRTUAL")
          .replaceAll(raw.projectRoot, "$PROJECT")
          .replaceAll(runtime.evidence.ownedRoot!, "$OWNED");
        return text;
      })
      .join("\n");
    fs.writeFileSync(path.join(directory, "normalized-default.txt"), normalized);
    results.push({
      role,
      normalized,
      binarySha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"),
      captures: names.length,
    });
  } finally {
    runtime.cleanup();
  }
}
assert.equal(
  results[1].normalized,
  results[0].normalized,
  "full default receipt bytes must remain equal after known path rebasing",
);
fs.writeFileSync(
  path.join(out, "acceptance.json"),
  JSON.stringify(
    {
      passed: true,
      scope:
        "public0.602 vs source default receipt; only known project/virtual/owned path prefixes normalized",
      rows: results.map(({ normalized, ...row }) => ({
        ...row,
        normalizedSha256: createHash("sha256").update(normalized).digest("hex"),
      })),
    },
    null,
    2,
  ),
);
console.log(JSON.stringify({ defaultByteGolden: true, captures: results[0].captures }));
