/** Real host matrix for #7004/#7005; direct host capability is recorded separately. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual, parseArgs } from "node:util";

import { formatPatinaMessage } from "../format.ts";
import type { PatinaBinding, PatinaDiagnostic } from "../model.ts";

interface OriginalCase {
  filename: string;
  source: string;
  sha256: string;
  config: string;
  origin: string;
  rule: string;
  anchors: { line: number; column: number; length: number }[];
  contrasts: { name: string; source: string; sha256: string; anchors: OriginalCase["anchors"] }[];
}
interface HostDiagnostic {
  code: string;
  message: string;
  severity: string;
  filename: string;
  labels: { span: { offset: number; length: number; line: number; column: number } }[];
}
interface MatrixOptions {
  pluginPackage: string;
  oxlintPackage: string;
  output: string;
  expectation: "candidate" | "public-0.458";
}
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const packetPath = fileURLToPath(new URL("./original-sfc-host-7004-7005.json", import.meta.url));
const originals = (JSON.parse(fs.readFileSync(packetPath, "utf8")) as { cases: OriginalCase[] })
  .cases;

export async function runOriginalSfcHostMatrix(options: MatrixOptions): Promise<void> {
  assert.ok(["candidate", "public-0.458"].includes(options.expectation));
  fs.mkdirSync(options.output, { recursive: true });
  const pluginManifestPath = path.join(options.pluginPackage, "package.json");
  const pluginManifest = JSON.parse(fs.readFileSync(pluginManifestPath, "utf8")) as {
    version: string;
    main: string;
    bin: Record<string, string>;
  };
  assert.equal(JSON.parse(fs.readFileSync(pluginManifestPath, "utf8")).name, "oxlint-plugin-vize");
  if (options.expectation === "public-0.458") assert.equal(pluginManifest.version, "0.458.0");
  const pluginEntry = path.resolve(options.pluginPackage, pluginManifest.main);
  const wrapper = path.resolve(options.pluginPackage, pluginManifest.bin["oxlint-vize"]!);
  const oxlintManifestPath = path.join(options.oxlintPackage, "package.json");
  const oxlintManifest = JSON.parse(fs.readFileSync(oxlintManifestPath, "utf8")) as {
    version: string;
    bin: Record<string, string>;
  };
  const oxlint = path.resolve(options.oxlintPackage, oxlintManifest.bin.oxlint!);
  const require = createRequire(pluginManifestPath);
  if (options.expectation === "public-0.458") {
    for (const key of PUBLIC_OVERRIDE_KEYS)
      assert.ok(!process.env[key], `public baseline cannot inherit${key}`);
    assert.equal(
      Object.values(require.cache).filter(
        (entry) =>
          entry?.filename.endsWith(".node") && typeof entry.exports?.lintPatinaSfc === "function",
      ).length,
      0,
      "public baseline must start in a fresh child without a loaded Vize image",
    );
  }
  await import(pathToFileURL(pluginEntry).href);
  const images = Object.values(require.cache).filter(
    (entry) =>
      entry?.filename.endsWith(".node") && typeof entry.exports?.lintPatinaSfc === "function",
  );
  assert.equal(
    images.length,
    1,
    "one actual native reference image must be attributable after importing this plugin",
  );
  const nativeImage = images[0]!;
  const imagePath = nativeImage.filename;
  const imageHash = sha256(fs.readFileSync(imagePath));
  const publicNative =
    options.expectation === "public-0.458" ? authenticatePublicNative(options, imagePath) : null;
  const binding = nativeImage.exports as PatinaBinding;
  const env = { ...process.env };
  delete env.GITHUB_ACTIONS;
  const receipt: Record<string, unknown> = {
    expectation: options.expectation,
    hostVersion: oxlintManifest.version,
    pluginVersion: pluginManifest.version,
    packet: { path: packetPath, sha256: sha256(fs.readFileSync(packetPath)) },
    plugin: { path: pluginEntry, sha256: sha256(fs.readFileSync(pluginEntry)) },
    manifests: [pluginManifestPath, oxlintManifestPath].map((file) => ({
      path: file,
      sha256: sha256(fs.readFileSync(file)),
    })),
    native: {
      path: imagePath,
      sha256: imageHash,
      authority:
        "actual parent native reference; same plugin resolution; no child PID instrumentation claim",
      publicNative,
    },
    removedEnvironment: ["GITHUB_ACTIONS"],
    cells: [],
  };
  const cells = receipt.cells as Record<string, unknown>[];
  const failures: unknown[] = [];
  const run = (cwd: string, cellName: string, entry: string, args: string[]) => {
    const directory = path.join(options.output, cellName);
    fs.mkdirSync(directory, { recursive: true });
    const stdoutPath = path.join(directory, "stdout.bin");
    const stderrPath = path.join(directory, "stderr.bin");
    const stdoutFd = fs.openSync(stdoutPath, "wx");
    const stderrFd = fs.openSync(stderrPath, "wx");
    let result;
    try {
      result = spawnSync(process.execPath, [entry, ...args], {
        cwd,
        env,
        stdio: ["ignore", stdoutFd, stderrFd],
        timeout: 120_000,
      });
    } finally {
      fs.closeSync(stdoutFd);
      fs.closeSync(stderrFd);
    }
    const stdout = fs.readFileSync(stdoutPath);
    const stderr = fs.readFileSync(stderrPath);
    const error =
      result.error == null
        ? null
        : Object.fromEntries(
            Object.getOwnPropertyNames(result.error).map((key) => [
              key,
              Reflect.get(result.error!, key),
            ]),
          );
    const cell = {
      name: cellName,
      argv: [process.execPath, entry, ...args],
      cwd,
      environment: {
        PATH: env.PATH,
        NODE_OPTIONS: env.NODE_OPTIONS,
        VIZE_PREFER_WORKSPACE_BINDING: env.VIZE_PREFER_WORKSPACE_BINDING,
      },
      status: result.status,
      signal: result.signal,
      error,
      stdout: { path: stdoutPath, bytes: stdout.length, sha256: sha256(stdout) },
      stderr: { path: stderrPath, bytes: stderr.length, sha256: sha256(stderr) },
    };
    cells.push(cell);
    fs.writeFileSync(path.join(directory, "process.json"), JSON.stringify(cell, null, 2) + "\n");
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return {
      cell,
      stdout: stdout.toString("utf8"),
      stderr: stderr.toString("utf8"),
      status: result.status,
    };
  };

  try {
    for (const original of originals) {
      assert.equal(sha256(original.source), original.sha256);
      const cwd = path.join(options.output, "projects", original.filename);
      fs.mkdirSync(path.join(cwd, "node_modules"), { recursive: true });
      fs.symlinkSync(
        options.pluginPackage,
        path.join(cwd, "node_modules/oxlint-plugin-vize"),
        "junction",
      );
      fs.symlinkSync(options.oxlintPackage, path.join(cwd, "node_modules/oxlint"), "junction");
      const file = path.join(cwd, original.filename);
      fs.writeFileSync(file, original.source, { flag: "wx" });
      fs.writeFileSync(path.join(cwd, ".oxlintrc.json"), original.config, { flag: "wx" });
      const native = binding.lintPatinaSfc(original.source, {
        filename: original.filename,
        enabledRules: [original.rule],
        preset: "incremental",
      });
      fs.writeFileSync(
        path.join(cwd, "native-reference.json"),
        JSON.stringify(native, null, 2) + "\n",
      );
      assert.deepEqual(
        native.diagnostics.map((diagnostic) => ({
          line: diagnostic.location.start.line,
          column: diagnostic.location.start.column,
          length: diagnostic.location.end.offset - diagnostic.location.start.offset,
        })),
        original.anchors,
      );
      const expectedSeverity = original.rule.startsWith("a11y/") ? "warning" : "error";
      const expectedStatus = expectedSeverity === "warning" ? 0 : 1;
      for (const [host, entry] of [
        ["direct", oxlint],
        ["wrapper", wrapper],
      ] as const) {
        for (const reporter of ["original-default", "json"] as const) {
          const cellName = `${original.filename}-${host}-${reporter}`;
          try {
            const args =
              reporter === "json" ? ["-f", "json", original.filename] : [original.filename];
            const result = run(cwd, cellName, entry, args);
            const raw = result.stdout + result.stderr;
            assert.doesNotMatch(raw, /RangeError|JSPluginError|JS plugin error/u);
            if (host === "wrapper")
              assert.doesNotMatch(raw, /node_modules[\\/]\.vize[\\/]oxlint-plugin-vize/u);
            if (reporter === "original-default") {
              // Preserve the complete original command/output independently.
              assert.equal(
                result.status,
                host === "direct" && original.filename === "TplOnly.vue" ? 0 : expectedStatus,
              );
              continue;
            }
            const report = JSON.parse(result.stdout) as { diagnostics: HostDiagnostic[] };
            const target = report.diagnostics.filter(
              (diagnostic) => diagnostic.code === `vize(${original.rule})`,
            );
            if (host === "direct" && original.filename === "TplOnly.vue" && target.length === 0) {
              Object.assign(result.cell, {
                capability: "no template-only Program dispatch observed",
              });
              assert.equal(result.status, 0);
              continue;
            }
            assert.equal(result.status, expectedStatus);
            assert.equal(target.length, native.diagnostics.length);
            for (const [index, diagnostic] of target.entries()) {
              const expected = native.diagnostics[index]!;
              const owner =
                original.filename === "MyPicker.vue" && options.expectation === "public-0.458"
                  ? ["<span>", "SFC"][index]!
                  : "<template>";
              assert.equal(
                diagnostic.message,
                formatPatinaMessage(expected, {
                  hasMappedLocation: host === "wrapper",
                  blockLabel: owner,
                  helpLevel: "full",
                }),
              );
              assert.equal(diagnostic.filename, original.filename);
              assert.equal(diagnostic.severity, expectedSeverity);
              assert.equal(diagnostic.labels.length, 1);
              const span = expectedSpan(original.source, expected);
              if (host === "wrapper") assert.deepEqual(diagnostic.labels[0]!.span, span);
              else
                Object.assign(result.cell, {
                  originalTemplateRangesSupported: target.every((entry, ordinal) =>
                    isDeepStrictEqual(
                      entry.labels[0]?.span,
                      expectedSpan(original.source, native.diagnostics[ordinal]!),
                    ),
                  ),
                });
            }
          } catch (error) {
            failures.push({ cell: cellName, error: String(error) });
          }
        }
      }
      const offConfig = JSON.parse(original.config) as { rules: Record<string, string> };
      offConfig.rules[`vize/${original.rule}`] = "off";
      fs.writeFileSync(
        path.join(cwd, ".disabled.json"),
        JSON.stringify(offConfig, null, 2) + "\n",
        { flag: "wx" },
      );
      for (const variant of original.contrasts) {
        assert.equal(sha256(variant.source), variant.sha256);
        fs.writeFileSync(path.join(cwd, `${variant.name}.vue`), variant.source, { flag: "wx" });
      }
      for (const control of [
        ...original.contrasts.map((variant) => ({
          name: variant.name,
          file: `${variant.name}.vue`,
          source: variant.source,
          anchors: variant.anchors,
          config: undefined,
        })),
        {
          name: "rule-disabled",
          file: original.filename,
          source: original.source,
          anchors: [],
          config: ".disabled.json",
        },
      ]) {
        for (const [host, entry] of [
          ["direct", oxlint],
          ["wrapper", wrapper],
        ] as const) {
          const cellName = `${original.filename}-${host}-${control.name}`;
          try {
            const args = [
              ...(control.config ? ["-c", control.config] : []),
              "-f",
              "json",
              control.file,
            ];
            const result = run(cwd, cellName, entry, args);
            const target = (
              JSON.parse(result.stdout) as { diagnostics: HostDiagnostic[] }
            ).diagnostics.filter((entry) => entry.code === `vize(${original.rule})`);
            assert.equal(
              target.length,
              host === "direct" && original.filename === "TplOnly.vue" ? 0 : control.anchors.length,
            );
            assert.equal(
              result.status,
              control.anchors.length === 0 || expectedSeverity === "warning" ? 0 : 1,
            );
            if (host === "wrapper" && !control.config) {
              const reference = binding.lintPatinaSfc(control.source, {
                filename: control.file,
                preset: "incremental",
                enabledRules: [original.rule],
              }).diagnostics;
              assert.deepEqual(
                reference.map((diagnostic) => ({
                  line: diagnostic.location.start.line,
                  column: diagnostic.location.start.column,
                  length: diagnostic.location.end.offset - diagnostic.location.start.offset,
                })),
                control.anchors,
              );
              assert.deepEqual(
                target.map((diagnostic) => diagnostic.filename),
                control.anchors.map(() => control.file),
              );
              assert.deepEqual(
                target.map((diagnostic) => diagnostic.labels[0]!.span),
                reference.map((diagnostic) => expectedSpan(control.source, diagnostic)),
              );
            }
          } catch (error) {
            failures.push({ cell: cellName, error: String(error) });
          }
        }
      }
      assert.equal(
        sha256(fs.readFileSync(file)),
        original.sha256,
        "the host must not mutate the original SFC",
      );
      assert.equal(fs.readFileSync(path.join(cwd, ".oxlintrc.json"), "utf8"), original.config);
    }
    assert.equal(sha256(fs.readFileSync(imagePath)), imageHash);
  } catch (error) {
    failures.push({ stage: "matrix setup or native reference", error: String(error) });
  } finally {
    receipt.failures = failures;
    receipt.contractAssertionsPassed = failures.length === 0;
    receipt.directHostCapabilities = cells
      .filter((cell) => String(cell.name).endsWith("-direct-json"))
      .map((cell) => ({
        name: cell.name,
        originalTemplateRangesSupported: cell.originalTemplateRangesSupported === true,
        capability: cell.capability ?? null,
      }));
    receipt.directHostAllOriginalCapabilitiesObserved =
      (receipt.directHostCapabilities as { originalTemplateRangesSupported: boolean }[]).length ===
        originals.length &&
      (receipt.directHostCapabilities as { originalTemplateRangesSupported: boolean }[]).every(
        (cell) => cell.originalTemplateRangesSupported,
      );
    receipt.scope =
      "Nested block ownership; known direct-host range/Program limitations are observations, not issue-completion credit.";
    fs.writeFileSync(
      path.join(options.output, "receipt.json"),
      JSON.stringify(receipt, null, 2) + "\n",
    );
  }
  assert.deepEqual(failures, [], `raw host evidence retained at${options.output}`);
}

export const PUBLIC_OVERRIDE_KEYS = [
  "NODE_OPTIONS",
  "NODE_PATH",
  "VIZE_OXLINT_NATIVE_CUSTODY",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_OXLINT_TEST_ENTRYPOINT",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
] as const;

function authenticatePublicNative(options: MatrixOptions, imagePath: string) {
  const root = fs.realpathSync(path.resolve(options.pluginPackage, "../.."));
  for (const [directory, name] of [
    [options.pluginPackage, "oxlint-plugin-vize"],
    [options.oxlintPackage, "oxlint"],
  ])
    assert.equal(
      fs.realpathSync(directory!),
      path.join(root, "node_modules", name!),
      "public host packages must belong to the same actual installation",
    );
  const relative = path.relative(root, fs.realpathSync(imagePath));
  assert.ok(
    !path.isAbsolute(relative) && relative.split(path.sep)[0] !== "..",
    "public native image cannot resolve outside its installation",
  );
  let directory = path.dirname(imagePath);
  while (!fs.existsSync(path.join(directory, "package.json"))) {
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory);
    assert.notEqual(directory, root);
    directory = parent;
  }
  const bytes = fs.readFileSync(path.join(directory, "package.json"));
  const manifest = JSON.parse(bytes.toString("utf8"));
  assert.ok(manifest.name.startsWith("@vizejs/native-"));
  assert.equal(manifest.version, "0.458.0");
  const lockBytes = fs.readFileSync(path.join(root, "package-lock.json"));
  const key = path.relative(root, directory).split(path.sep).join("/");
  const locked = JSON.parse(lockBytes.toString("utf8")).packages[key];
  assert.equal(locked.version, "0.458.0");
  assert.match(locked.integrity, /^sha512-/u);
  assert.ok(locked.resolved.startsWith("https://registry.npmjs.org/"));
  fs.writeFileSync(path.join(options.output, "public-native-package.json"), bytes, { flag: "wx" });
  fs.writeFileSync(path.join(options.output, "public-package-lock.json"), lockBytes, {
    flag: "wx",
  });
  return {
    root,
    package: manifest.name,
    version: manifest.version,
    manifestSha256: sha256(bytes),
    lockKey: key,
    lockSha256: sha256(lockBytes),
    integrity: locked.integrity,
    resolved: locked.resolved,
  };
}

function expectedSpan(source: string, diagnostic: PatinaDiagnostic) {
  const { start, end } = diagnostic.location;
  // Patina offsets are original UTF-8 bytes, and columns already UTF-16.
  assert.equal(
    Buffer.from(source).subarray(start.offset, end.offset).toString("utf8").length > 0,
    true,
  );
  return {
    offset: start.offset,
    length: end.offset - start.offset,
    line: start.line,
    column: start.column,
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { values } = parseArgs({
    options: {
      "plugin-package": { type: "string" },
      "oxlint-package": { type: "string" },
      output: { type: "string" },
      expectation: { type: "string" },
    },
  });
  assert.ok(values["plugin-package"] && values["oxlint-package"] && values.output);
  assert.ok(values.expectation === "candidate" || values.expectation === "public-0.458");
  await runOriginalSfcHostMatrix({
    pluginPackage: path.resolve(values["plugin-package"]),
    oxlintPackage: path.resolve(values["oxlint-package"]),
    output: path.resolve(values.output),
    expectation: values.expectation,
  });
}
