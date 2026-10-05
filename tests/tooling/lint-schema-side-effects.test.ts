import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = path.join(root, "tests/_fixtures/cli/lint-schema-side-effects");
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const localSchema = "node_modules/.vize/vize.config.schema.json";
const schemaBytes = fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"));
type TreeRow = { path: string; kind: string; bytes?: number[]; sha256?: string; target?: string };

function inventory(directory: string): TreeRow[] {
  const rows: TreeRow[] = [];
  function visit(relative: string) {
    const absolute = path.join(directory, relative);
    const stat = fs.lstatSync(absolute);
    if (stat.isSymbolicLink()) {
      rows.push({ path: relative, kind: "symlink", target: fs.readlinkSync(absolute) });
    } else if (stat.isDirectory()) {
      rows.push({ path: relative, kind: "directory" });
      for (const name of fs.readdirSync(absolute).sort()) visit(path.posix.join(relative, name));
    } else {
      const bytes = fs.readFileSync(absolute);
      rows.push({ path: relative, kind: "file", bytes: [...bytes], sha256: sha256(bytes) });
    }
  }
  visit("");
  return rows.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

type Scenario = {
  id: string;
  config?: Record<string, unknown>;
  installed?: boolean;
  explicit?: boolean;
  noConfig?: boolean;
  nested?: boolean;
  generate?: boolean;
  seed?: "stale" | "current";
  obstruction?: "dependencies-file" | "dependencies-link" | "cache-link" | "schema-link";
  check?: boolean;
  parseError?: string;
};
const scenarios: Scenario[] = [
  { id: "original-no-config", noConfig: true },
  { id: "default-no-config-file" },
  { id: "implicit-json", config: {} },
  { id: "explicit-json", config: {}, explicit: true },
  { id: "installed-plain-json", config: {}, installed: true },
  { id: "explicit-reference-uninstalled", config: { $schema: localSchema }, explicit: true },
  { id: "implicit-reference-uninstalled", config: { $schema: localSchema } },
  {
    id: "explicit-reference-installed",
    config: { $schema: localSchema },
    installed: true,
    explicit: true,
    generate: true,
  },
  {
    id: "implicit-reference-installed",
    config: { $schema: `./${localSchema}` },
    installed: true,
    generate: true,
  },
  {
    id: "stale-schema",
    config: { $schema: localSchema },
    installed: true,
    seed: "stale",
    generate: true,
  },
  {
    id: "current-schema",
    config: { $schema: localSchema },
    installed: true,
    seed: "current",
    generate: true,
  },
  {
    id: "no-config-ignores-reference",
    config: { $schema: localSchema },
    installed: true,
    noConfig: true,
    seed: "stale",
  },
  {
    id: "remote-reference",
    config: { $schema: "https://example.test/schema.json" },
    installed: true,
  },
  { id: "other-local-reference", config: { $schema: "custom.schema.json" }, installed: true },
  {
    id: "nonstring-reference",
    config: { $schema: true },
    installed: true,
    parseError: "invalid type: boolean `true`, expected a string at line 2 column 17",
  },
  {
    id: "relocated-config",
    config: { $schema: localSchema },
    installed: true,
    explicit: true,
    nested: true,
    generate: true,
  },
  ...(["dependencies-file", "dependencies-link", "cache-link", "schema-link"] as const).map(
    (obstruction) => ({
      id: obstruction,
      config: { $schema: localSchema },
      installed: true,
      obstruction,
    }),
  ),
  {
    id: "disabled-check-plain",
    check: true,
    config: { typeChecker: { enabled: false } },
    explicit: true,
  },
  {
    id: "disabled-check-reference",
    check: true,
    config: { $schema: localSchema, typeChecker: { enabled: false } },
    installed: true,
    generate: true,
  },
];

test("source CLI preserves original lint output and only materializes explicitly authored installed schemas", () => {
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, build);
  const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "manifest.json"), "utf8"));
  for (const entry of manifest.files) {
    const bytes = fs.readFileSync(path.join(fixture, entry.path));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(sha256(bytes), entry.sha256);
  }
  const original = fs.readFileSync(path.join(fixture, "MyText.vue.txt"));
  assert.deepEqual(original, Buffer.from("<template>\n  <p>x</p>\n</template>\n"));
  const output = path.join(root, "target/differential/cli/lint-schema-side-effects.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const rows: unknown[] = [];
  const record = {
    sourceBuild: receipt,
    manifest,
    scenarios: scenarios.length,
    rows,
    status: "PENDING",
  };
  const save = () => fs.writeFileSync(output, `${JSON.stringify(record, null, 2)}\n`);
  const env = { ...process.env, NO_COLOR: "1" };
  for (const key of ["FORCE_COLOR", "CLICOLOR_FORCE"]) delete env[key];
  try {
    for (const scenario of scenarios) {
      const cwd = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-7986-")));
      const outside = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-7986-outside-")));
      try {
        fs.writeFileSync(path.join(cwd, "MyText.vue"), original);
        fs.writeFileSync(path.join(outside, "sentinel"), "authored external bytes\n");
        fs.writeFileSync(path.join(outside, "vize.config.schema.json"), "external stale schema\n");
        const configDir = scenario.nested ? path.join(cwd, "config") : cwd;
        fs.mkdirSync(configDir, { recursive: true });
        const configPath = path.join(configDir, "vize.config.json");
        if (scenario.config)
          fs.writeFileSync(configPath, `${JSON.stringify(scenario.config, null, 2)}\n`);
        const dependencies = path.join(configDir, "node_modules");
        const schema = path.join(configDir, localSchema);
        if (scenario.obstruction === "dependencies-file")
          fs.writeFileSync(dependencies, "not a directory\n");
        else if (scenario.obstruction === "dependencies-link")
          fs.symlinkSync(outside, dependencies, process.platform === "win32" ? "junction" : "dir");
        else if (scenario.installed) fs.mkdirSync(dependencies);
        if (scenario.obstruction === "cache-link")
          fs.symlinkSync(
            outside,
            path.dirname(schema),
            process.platform === "win32" ? "junction" : "dir",
          );
        if (scenario.obstruction === "schema-link") {
          fs.mkdirSync(path.dirname(schema));
          fs.symlinkSync(path.join(outside, "vize.config.schema.json"), schema, "file");
        }
        if (scenario.seed) {
          fs.mkdirSync(path.dirname(schema), { recursive: true });
          fs.writeFileSync(schema, scenario.seed === "current" ? schemaBytes : Buffer.from("{}\n"));
        }
        const before = inventory(cwd);
        const externalBefore = inventory(outside);
        const initialRevision =
          scenario.seed === "current"
            ? fs.statSync(schema, { bigint: true }).mtimeNs.toString()
            : null;
        for (const format of ["plain", "json"]) {
          const args = scenario.check
            ? ["check", "--format", "json"]
            : ["lint", "-f", format, "MyText.vue"];
          if (scenario.noConfig) args.push("--no-config");
          if (scenario.explicit) args.push("--config", path.relative(cwd, configPath));
          const result = spawnSync(binary, args, { cwd, env, maxBuffer: 8 * 1024 * 1024 });
          const row = {
            id: scenario.id,
            format,
            command: binary,
            args,
            cwd,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdout: result.stdout?.toString("utf8") ?? "",
            stderr: result.stderr?.toString("utf8") ?? "",
            stdoutBytes: [...(result.stdout ?? [])],
            stderrBytes: [...(result.stderr ?? [])],
            before,
            after: inventory(cwd),
            externalBefore,
            externalAfter: inventory(outside),
          };
          rows.push(row);
          save();
          assert.equal(row.error, null, scenario.id);
          assert.equal(row.signal, null, scenario.id);
          assert.equal(
            row.status,
            scenario.parseError ? 2 : 0,
            `${scenario.id}: ${row.stdout}${row.stderr}`,
          );
          assert.deepEqual(Buffer.from(row.stdout), result.stdout);
          assert.deepEqual(Buffer.from(row.stderr), result.stderr);
          if (scenario.parseError) {
            assert.equal(row.stdout, "");
            assert.equal(
              row.stderr,
              `\x1b[31mError:\x1b[0m failed to parse ${configPath}: ${scenario.parseError}\n`,
            );
          } else if (scenario.check) {
            assert.equal(row.stdout, "");
            assert.equal(
              row.stderr,
              "[vize] Skipping check because typeChecker.enabled is false in vize.config.\n",
            );
          } else {
            assert.equal(row.stderr, "");
            assert.equal(
              row.stdout,
              format === "plain"
                ? "Patina lint report: No problems found in 1 file(s)\n"
                : JSON.stringify(
                    [{ file: "MyText.vue", messages: [], errorCount: 0, warningCount: 0 }],
                    null,
                    2,
                  ),
            );
          }
          assert.deepEqual(row.externalAfter, externalBefore, `${scenario.id}: external mutation`);
          if (scenario.generate) {
            assert.deepEqual(fs.readFileSync(schema), schemaBytes);
            const expected = before.filter(
              (entry) => entry.path !== path.relative(cwd, schema).split(path.sep).join("/"),
            );
            const cache = path.relative(cwd, path.dirname(schema)).split(path.sep).join("/");
            if (!expected.some((entry) => entry.path === cache))
              expected.push({ path: cache, kind: "directory" });
            expected.push({
              path: `${cache}/vize.config.schema.json`,
              kind: "file",
              bytes: [...schemaBytes],
              sha256: sha256(schemaBytes),
            });
            expected.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
            assert.deepEqual(row.after, expected, `${scenario.id}: unexpected mutation`);
            if (initialRevision !== null)
              assert.equal(
                fs.statSync(schema, { bigint: true }).mtimeNs.toString(),
                initialRevision,
              );
          } else
            assert.deepEqual(row.after, before, `${scenario.id}: read-only command mutated tree`);
        }
      } finally {
        fs.rmSync(cwd, { recursive: true, force: true });
        fs.rmSync(outside, { recursive: true, force: true });
      }
    }
    assert.equal(rows.length, 44);
    record.status = "PASS";
  } catch (error) {
    record.status = "FAIL";
    throw error;
  } finally {
    save();
  }
});
