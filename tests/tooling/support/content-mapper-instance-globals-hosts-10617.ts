import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  compilerRows,
  expectedHost,
  location,
  sha,
  type GlobalCase,
  type Packet,
} from "./content-mapper-instance-globals-10617.ts";

export type RunHost = (label: string, command: string, args: string[], cwd: string) => Packet;
export type AdditionalHost = { id: string; command: string };

export function admitAdditionalHosts(
  output: string,
  files: Map<string, string>,
  published: string,
  repoRoot: string,
  run: RunHost,
) {
  const authority = JSON.parse(files.get("host-authority.json")!);
  assert.equal(authority.schema, "vize.instance-globals-host-authority");
  const nightly = path.join(output, "official-nightly");
  fs.mkdirSync(nightly, { recursive: true });
  for (const name of ["package.json", "package-lock.json"])
    fs.writeFileSync(path.join(nightly, name), files.get(`official-host/${name}.txt`)!);
  const installed = run(
    "official-nightly/locked-install",
    process.platform === "win32" ? "npm.cmd" : "npm",
    [
      "ci",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--prefer-offline",
      `--registry=${authority.officialNightly.registry}`,
    ],
    nightly,
  );
  assert.equal(installed.status, 0);
  const identities = [
    [
      "reported-beta",
      published,
      "7.1.0-beta",
      JSON.parse(files.get("project/package-lock.json.txt")!),
    ],
    [
      "official-nightly",
      nightly,
      authority.officialNightly.version,
      JSON.parse(files.get("official-host/package-lock.json.txt")!),
    ],
  ].map(([id, root, expectedVersion, lock]) => {
    const directory = root as string;
    const version = expectedVersion as string;
    const locked = lock as { packages: Record<string, { version: string; integrity: string }> };
    const command = path.join(directory, "node_modules/typescript/bin/tsc");
    const nativeName = `@typescript/typescript-${process.platform}-${process.arch}`;
    const native = path.join(
      directory,
      "node_modules",
      nativeName,
      "lib",
      process.platform === "win32" ? "tsc.exe" : "tsc",
    );
    const manifest = JSON.parse(
      fs.readFileSync(path.join(directory, "node_modules/typescript/package.json"), "utf8"),
    );
    assert.equal(manifest.version, version);
    assert.equal(locked.packages[`node_modules/${nativeName}`].version, version);
    if (id === "official-nightly")
      assert.equal(
        locked.packages["node_modules/typescript"].integrity,
        authority.officialNightly.integrity,
      );
    const packet = run(`${id}/admitted-version`, command, ["--version"], directory);
    assert.deepEqual(
      {
        status: packet.status,
        signal: packet.signal,
        error: packet.error,
        stdout: packet.stdout,
        stderr: packet.stderr,
      },
      { status: 0, signal: null, error: null, stdout: `Version ${version}\n`, stderr: "" },
    );
    const bytes = fs.readFileSync(native);
    return {
      id,
      version,
      command,
      launcherSha256: sha(fs.readFileSync(command)),
      native: fs.realpathSync(native),
      nativeSha256: sha(bytes),
      nativeBytes: bytes.length,
      platformIntegrity: locked.packages[`node_modules/${nativeName}`].integrity,
      versionPacket: packet,
    };
  });
  const hosts: AdditionalHost[] = [
    { id: "official-nightly", command: path.join(nightly, "node_modules/typescript/bin/tsc") },
  ];
  const requested = process.env.VIZE_TEST_GLOBALS_SOURCE_HOST;
  const receiptFile = process.env.VIZE_TEST_GLOBALS_SOURCE_HOST_RECEIPT;
  const required = process.env.VIZE_TEST_GLOBALS_HOST_MATRIX_REQUIRED === "1";
  if (required)
    assert.ok(
      requested && receiptFile,
      "the dedicated source-host matrix cannot silently omit its pinned source host",
    );
  assert.equal(
    Boolean(requested),
    Boolean(receiptFile),
    "source host and receipt must be supplied together",
  );
  let sourceHost: unknown = {
    status: "unverified",
    reason: "pinned source host not configured in this runtime; no credit",
  };
  if (requested && receiptFile) {
    const receipt = JSON.parse(fs.readFileSync(receiptFile, "utf8"));
    const binary = fs.realpathSync(requested);
    const bytes = fs.readFileSync(binary);
    const version = run("pinned-source/admitted-version", binary, ["--version"], repoRoot);
    const source = run("pinned-source/checked-out-source", "git", ["rev-parse", "HEAD"], repoRoot);
    assert.equal(source.status, 0);
    assert.equal(source.stderr, "");
    assert.equal(version.status, 0);
    assert.equal(version.stderr, "");
    assert.match(version.stdout, /^Version [^\r\n]+\n$/u);
    const expected = {
      schema: "vize.instance-globals-source-host",
      version: 1,
      repository: "microsoft/TypeScript",
      upstream: "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
      source: source.stdout.trim(),
      run: process.env.GITHUB_RUN_ID ?? receipt.run,
      binary,
      sha256: sha(bytes),
      size: bytes.length,
      compilerVersion: {
        status: version.status,
        signal: version.signal,
        error: version.error,
        stdout: version.stdout,
        stderr: version.stderr,
      },
    };
    assert.equal(authority.sourceHost.repository, expected.repository);
    assert.equal(authority.sourceHost.revision, expected.upstream);
    assert.match(expected.source, /^[a-f0-9]{40}$/u);
    assert.match(expected.run, /^\d+$/u);
    assert.deepEqual(receipt, expected, "exact pinned source-host receipt");
    sourceHost = { status: "admitted", receipt, receiptSha256: sha(fs.readFileSync(receiptFile)) };
    hosts.push({ id: "pinned-source", command: binary });
  }
  fs.writeFileSync(
    path.join(output, "host-matrix.json"),
    `${JSON.stringify({ authority, required, identities, sourceHost }, null, 2)}\n`,
  );
  return {
    hosts,
    sourceHostAdmitted: Boolean(requested && receiptFile),
    required,
    cleanupRoot: nightly,
  };
}

export function assertHostCase(
  content: string,
  entry: GlobalCase,
  context: Packet,
  after: Packet,
  cli: Packet,
  compilerOptions: unknown,
  label: string,
) {
  assertMapperCase(content, entry, context, after, label);
  assertNativeCase(content, entry, context, cli, compilerOptions, label);
}

export function assertMapperCase(
  content: string,
  entry: GlobalCase,
  context: Packet,
  after: Packet,
  label: string,
) {
  const rows = compilerRows(context.stdout);
  assert.deepEqual(
    {
      status: context.status,
      signal: context.signal,
      error: context.error,
      stderr: context.stderr,
    },
    { status: rows.length ? 2 : 0, signal: null, error: null, stderr: "" },
    label,
  );
  assert.ok(
    rows.every((row) => row.file === "oracle/Context.ts"),
    "independent diagnostics must belong to the handwritten context",
  );
  const expected = expectedHost(content, entry, rows);
  assert.deepEqual(
    {
      status: after.status,
      signal: after.signal,
      error: after.error,
      stdout: after.stdout,
      stderr: after.stderr,
    },
    { status: expected ? 2 : 0, signal: null, error: null, stdout: expected, stderr: "" },
    label,
  );
}

export function assertNativeCase(
  content: string,
  entry: GlobalCase,
  context: Packet,
  cli: Packet,
  compilerOptions: unknown,
  label: string,
  templateDisabled = false,
) {
  const rows = compilerRows(context.stdout);
  assert.equal(rows.length, entry.errors.length, label);
  assert.deepEqual(
    {
      status: context.status,
      signal: context.signal,
      error: context.error,
      stderr: context.stderr,
    },
    { status: rows.length ? 2 : 0, signal: null, error: null, stderr: "" },
    label,
  );
  const nativeErrors = (templateDisabled ? [] : entry.errors)
    .map((error, index) => ({ error, row: rows[index] }))
    .filter(({ error }) => !error.suppressed)
    .map(({ error, row }) => ({
      at: location(content, error.anchor, error.token),
      code: row.code,
      message: row.message,
      binding: entry.binding,
    }));
  if (entry.unusedDirective && !templateDisabled)
    nativeErrors.push({
      at: location(content, "<!-- @vue-expect-error -->", "<!--"),
      code: 2578,
      message: "Unused '@vue-expect-error' directive.",
      binding: "@vue-expect-error",
    });
  // The CLI presents authored locations in source order; no actual packet is sorted or changed.
  nativeErrors.sort((a, b) => a.at.line - b.at.line || a.at.column - b.at.column);
  const diagnostics = nativeErrors.map(({ at, code, message, binding }) => {
    const line = content.split(/\r?\n/u)[at.line - 1].trim();
    return `error:${at.line}:${at.column} [TS${code}] ${message} (source: ${line.length > 160 ? `${line.slice(0, 157)}...` : line}${binding ? `; binding: ${binding}` : ""})`;
  });
  const report = {
    files: [
      { file: "src/Globals.vue", diagnostics },
      { file: "src/globals.d.ts", diagnostics: [] },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions,
        files: ["src/Globals.vue", "src/globals.d.ts"],
      },
    ],
    errorCount: diagnostics.length,
    warningCount: 0,
    fileCount: 2,
  };
  assert.deepEqual(
    { status: cli.status, signal: cli.signal, error: cli.error, stderr: cli.stderr },
    { status: diagnostics.length ? 1 : 0, signal: null, error: null, stderr: "" },
    label,
  );
  assert.deepEqual(JSON.parse(cli.stdout), report, `whole native report ${label}`);
}

export function assertPublishedControl(
  content: string,
  entry: GlobalCase,
  context: Packet,
  before: Packet,
  label: string,
) {
  const rows = compilerRows(context.stdout);
  const expectedBefore = rows
    .flatMap((row, index) => {
      const error = entry.errors[index];
      if (error.kind === "undeclared") return [];
      const at = location(content, error.anchor, error.token);
      return [`src/Globals.vue(${at.line},${at.column}): error TS${row.code}: ${row.message}\n`];
    })
    .join("");
  assert.deepEqual(
    {
      status: before.status,
      signal: before.signal,
      error: before.error,
      stdout: before.stdout,
      stderr: before.stderr,
    },
    {
      status: expectedBefore ? 2 : 0,
      signal: null,
      error: null,
      stdout: expectedBefore,
      stderr: "",
    },
    `faithful published red/control ${label}`,
  );
}
