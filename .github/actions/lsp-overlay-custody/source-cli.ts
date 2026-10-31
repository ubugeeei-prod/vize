import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

type Row = Record<string, unknown>;
const test =
  "lsp_cli::overlay_storage::default_native_overlay_preserves_the_whole_linked_installation_and_authored_project";
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function row(value: unknown): Row {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Row;
}
function list(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
function text(value: unknown): string {
  assert.equal(typeof value, "string");
  return value as string;
}
function image(file: string): Row {
  assert.ok(fs.lstatSync(file).isFile());
  const bytes = fs.readFileSync(file);
  return { path: fs.realpathSync(file), bytes: bytes.length, sha256: hash(bytes) };
}
function execution(stdout: string, expected: string[]): void {
  const announced = [...stdout.matchAll(/^test (\S+) \.\.\./gm)].map((match) => match[1]!);
  assert.deepEqual(announced.sort(), [...expected].sort());
  const summaries = [
    ...stdout.matchAll(
      /test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g,
    ),
  ];
  assert.equal(summaries.length, 1);
  assert.deepEqual(summaries[0]!.slice(1), [String(expected.length), "0", "0", "0"]);
}
function sourceTests(file: string, prefix: string): string[] {
  const source = fs.readFileSync(file, "utf8");
  return [...source.matchAll(/((?:#\[[^\]]+\]\s*)+)fn ([a-z_]+)\(/g)]
    .filter(
      (match) =>
        match[1]!.includes("#[test]") &&
        !(match[1]!.includes("#[cfg(windows)]") && process.platform !== "win32") &&
        !(match[1]!.includes("#[cfg(unix)]") && process.platform === "win32"),
    )
    .map((match) => prefix + match[2]!);
}
function libraryExecution(capture: string, nativeSha: string): void {
  const groups = [
    [
      "storage",
      "crates/vize_canon/src/lsp_client/session_storage/tests.rs",
      "lsp_client::session_storage::tests::",
    ],
    [
      "close",
      "crates/vize_canon/src/lsp_client/session/close/tests.rs",
      "lsp_client::session::close::tests::",
    ],
    [
      "logical",
      "crates/vize_canon/src/lsp_client/session/logical.rs",
      "lsp_client::session::logical::tests::",
    ],
    ...(process.platform === "linux"
      ? [
          [
            "leases",
            "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests.rs",
            "corsa_bridge::bridge::initial_diagnostics::tests::",
          ],
        ]
      : []),
  ];
  for (const [name, file, prefix] of groups) {
    const expected = sourceTests(file!, prefix!);
    assert.ok(expected.length > 0);
    execution(fs.readFileSync(path.join(capture, `${name}.stdout`), "utf8"), expected);
  }
  assert.equal(fs.readFileSync(path.join(capture, "native.status"), "utf8").trim(), "0");
  const nativeStdout = fs.readFileSync(path.join(capture, "native.stdout"), "utf8");
  execution(nativeStdout, [
    "lsp_client::session_storage::native_tests::actual_native_old_and_private_roots_preserve_complete_configured_overlay_answers",
  ]);
  const markers = [...nativeStdout.matchAll(/^native-overlay-vue-only-control (.+)$/gm)];
  assert.equal(markers.length, 1);
  assert.deepEqual(JSON.parse(markers[0]![1]!), {
    schema: 1,
    phases: ["valid", "dependency-error", "dependency-repair", "dependency-close"],
    wholeDiagnosticsMatched: 4,
    nativeSha256: nativeSha,
  });
}
function main(): void {
  const root = fs.realpathSync(process.cwd());
  const capture = process.env.VIZE_PRIVATE_OVERLAY_CAPTURE;
  const source = process.env.EXPECTED_SOURCE;
  assert.ok(capture && path.isAbsolute(capture));
  assert.ok(source && /^[a-f0-9]{40}$/.test(source));
  assert.ok(!process.env.CARGO_TARGET_DIR, "ordinary checkout target is required");
  const git = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
  assert.equal(git.status, 0);
  assert.equal(git.stdout.trim(), source);
  assert.ok(
    !process.env.VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS ||
      process.env.VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS === "10000",
    "foreign test-parent diagnostic timeout refused",
  );
  const provider = row(
    JSON.parse(fs.readFileSync("target/nextest/full/native-runtime-provider.json", "utf8")),
  );
  assert.equal(provider.sourceSha, source);
  assert.equal(provider.selection, "checkout-locked-platform-package");
  assert.equal(
    row(provider.package).name,
    `@typescript/typescript-${process.platform}-${process.arch}`,
  );
  assert.equal(row(provider.package).version, "7.0.2");
  const native = row(provider.executable);
  assert.equal(fs.realpathSync(text(process.env.CORSA_PATH)), native.path);
  assert.equal(image(text(native.path)).sha256, native.sha256);
  assert.deepEqual(row(native.version), {
    args: ["--version"],
    stdout: "Version 7.0.2\n",
    stderr: "",
    exitCode: 0,
    signal: null,
  });

  function save(name: string, value: unknown): void {
    fs.writeFileSync(path.join(capture!, `${name}.json`), `${JSON.stringify(value, null, 2)}\n`, {
      flag: "wx",
      mode: 0o600,
    });
  }
  function run(name: string, executable: string, args: string[], env = process.env): Row {
    const stdout = fs.openSync(path.join(capture!, `${name}.stdout`), "wx", 0o600);
    const stderr = fs.openSync(path.join(capture!, `${name}.stderr`), "wx", 0o600);
    try {
      const result = spawnSync(executable, args, {
        cwd: root,
        env,
        stdio: ["ignore", stdout, stderr],
      });
      const outcome = {
        executable,
        args,
        cwd: root,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
      };
      save(`${name}.exit`, outcome);
      return outcome;
    } finally {
      fs.closeSync(stdout);
      fs.closeSync(stderr);
    }
  }
  function passed(value: Row): void {
    assert.equal(value.status, 0);
    assert.equal(value.signal, null);
    assert.equal(value.error, null);
  }
  function cargoRows(name: string): Row[] {
    return fs
      .readFileSync(path.join(capture!, `${name}.stdout`), "utf8")
      .split("\n")
      .filter(Boolean)
      .map((line) => row(JSON.parse(line)));
  }
  const cliPath = path.join(
    root,
    "target",
    "ci",
    process.platform === "win32" ? "vize.exe" : "vize",
  );
  const buildArgs = [
    "build",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize",
    "--message-format=json-render-diagnostics",
  ];
  passed(run("source-cli-build", "cargo", buildArgs));
  const artifacts = cargoRows("source-cli-build");
  const cli = artifacts.filter(
    (entry) =>
      entry.reason === "compiler-artifact" &&
      row(entry.target).name === "vize" &&
      entry.executable === cliPath,
  );
  assert.equal(cli.length, 1);
  assert.deepEqual(row(cli[0]!.target).kind, ["bin"]);
  assert.equal(row(cli[0]!.profile).test, false);
  assert.ok(
    !list(cli[0]!.features).includes("profiling") && !list(cli[0]!.features).includes("legacy"),
  );
  const maestro = artifacts.filter(
    (entry) => entry.reason === "compiler-artifact" && row(entry.target).name === "vize_maestro",
  );
  assert.ok(
    maestro.length > 0 &&
      maestro.every(
        (entry) =>
          !list(entry.features).includes("profiling") &&
          !list(entry.features).includes("micro-profiling"),
      ),
  );
  assert.ok(artifacts.some((entry) => entry.reason === "build-finished" && entry.success === true));
  const first = image(cliPath);
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-overlay-first-cli-"));
  const preserved = path.join(owner, "first-image");
  fs.copyFileSync(cliPath, preserved, fs.constants.COPYFILE_EXCL);
  try {
    passed(run("source-cli-first-version", cliPath, ["--version"]));
    assert.equal(fs.statSync(path.join(capture, "source-cli-first-version.stderr")).size, 0);
    assert.equal(image(cliPath).sha256, first.sha256);
    save("source-cli-first", {
      source,
      workflowEventSha: process.env.GITHUB_SHA ?? null,
      recipe: "cargo build --locked --profile ci -p vize",
      args: buildArgs,
      artifact: cli[0],
      maestro,
      image: first,
    });
    const args = [
      "test",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize",
      "--test",
      "integration",
      test,
      "--no-run",
      "--message-format=json-render-diagnostics",
    ];
    passed(run("source-cli-library-build", "cargo", args));
    const tests = cargoRows("source-cli-library-build").filter(
      (entry) =>
        entry.reason === "compiler-artifact" &&
        row(entry.target).name === "integration" &&
        typeof entry.executable === "string",
    );
    assert.equal(tests.length, 1);
    assert.equal(row(tests[0]!.profile).test, true);
    const postLibrary = image(cliPath);
    fs.copyFileSync(preserved, cliPath);
    fs.chmodSync(cliPath, 0o700);
    assert.equal(image(cliPath).sha256, first.sha256);
    save("source-cli-context", {
      source,
      firstCLI: first,
      postLibraryCLI: postLibrary,
      restoredCLI: image(cliPath),
      libraryImage: image(text(tests[0]!.executable)),
      libraryArtifact: tests[0],
      sameImageUnitCredit: false,
      diagnosticTimeoutMs: "10000",
      nativeProvider: provider,
    });
    const outcome = run(
      "source-cli-test",
      text(tests[0]!.executable),
      ["--exact", test, "--ignored", "--nocapture"],
      {
        ...process.env,
        VIZE_OVERLAY_EXPECTED_CLI_SHA: text(first.sha256),
        VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS: "10000",
      },
    );
    save("source-cli-after", {
      source,
      image: image(cliPath),
      diagnosticTimeoutMs: "10000",
      outcome,
    });
    assert.equal(image(cliPath).sha256, first.sha256);
    passed(outcome);
    const stdout = fs.readFileSync(path.join(capture, "source-cli-test.stdout"), "utf8");
    execution(stdout, [test]);
    libraryExecution(capture, text(native.sha256));
  } finally {
    fs.rmSync(owner, { recursive: true, force: false });
  }
}
if (import.meta.main) main();
