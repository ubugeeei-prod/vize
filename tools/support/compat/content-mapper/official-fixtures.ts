import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { readArchive, observeInstalledArchive } from "../../release/consumer_recovery/archive.ts";
import { publicInputs } from "../../release/public_acceptance/inputs.ts";

export interface Original {
  issue: number;
  files: Record<string, string>;
  baselineCodes: number[];
  expectedCodes: number[];
  expectedTerms: string[];
  baselineTerms: string[];
  baselineStdout?: string;
  baselineStderr?: string;
  expectedStdout?: string;
  expectedStderr?: string;
  baselineStatus?: number;
  expectedStatus?: number;
}
export interface Case {
  name: string;
  original: number;
  files: Record<string, string>;
  codes: number[];
  terms?: string[];
  build?: boolean;
  stdout?: string;
  stderr?: string;
  status?: number;
}
export interface Fixture {
  typescript: string;
  vue: string;
  baselineVize: string;
  originals: Original[];
  controls: Case[];
}

/** Parse selectors without starting a package install or compiler process. */
export function officialRunOptions(args: string[], root: string, defaultFixture: string) {
  const values = new Map<string, string>();
  const flags = new Set<string>();
  const explicit: string[] = [];
  for (let index = 0; index < args.length; index++) {
    const name = args[index];
    if (["--baseline-only", "--candidate-only", "--explicit-fixtures-only"].includes(name)) {
      assert.ok(!flags.has(name), `duplicate option ${name}`);
      flags.add(name);
      continue;
    }
    assert.ok(
      ["--output", "--public-version", "--public-inputs", "--fixture"].includes(name),
      `unknown option ${name}`,
    );
    const value = args[++index];
    assert.ok(value && !value.startsWith("--"), `${name} requires a value`);
    if (name === "--fixture") explicit.push(path.resolve(root, value));
    else {
      assert.ok(!values.has(name), `duplicate option ${name}`);
      values.set(name, value);
    }
  }
  const candidateOnly = flags.has("--candidate-only");
  const explicitOnly = flags.has("--explicit-fixtures-only");
  assert.equal(candidateOnly, explicitOnly, "public-only mode requires both explicit selectors");
  const publicVersion = values.get("--public-version");
  const publicInputs = values.get("--public-inputs");
  if (candidateOnly) {
    assert.ok(!flags.has("--baseline-only"), "conflicting baseline and candidate modes");
    assert.match(publicVersion ?? "", /^0\.[1-9][0-9]*\.0$/u);
    assert.ok(publicInputs, "public-only mode requires exact publication inputs");
    assert.ok(explicit.length, "public-only mode requires explicit nonempty fixtures");
    for (const file of explicit) {
      const relative = path.relative(root, file);
      assert.ok(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
    }
  } else assert.equal(publicInputs, undefined, "publication inputs require public-only mode");
  const fixturePaths = explicitOnly ? explicit : [defaultFixture, ...explicit];
  assert.equal(new Set(fixturePaths).size, fixturePaths.length, "duplicate fixture path");
  return {
    candidateOnly,
    baselineOnly: flags.has("--baseline-only"),
    publicVersion,
    publicInputs,
    fixturePaths,
    output: values.get("--output"),
  };
}

export function officialActorInputs(value: Record<string, string | undefined>) {
  const subject = publicInputs(value);
  const actor = value.PUBLIC_ACTOR ?? "";
  assert.match(actor, /^[0-9a-f]{40}$/u, "exact trusted tooling actor F required");
  return { subject, actor };
}

function surplusDefaultsPublicFixture(source: {
  issue: number;
  cases: Pick<Case, "name" | "files" | "codes">[];
}): Fixture {
  assert.equal(source.issue, 9499);
  assert.ok(Array.isArray(source.cases) && source.cases.length > 0);
  assert.equal(source.cases[0].name, "inline-surplus");
  const config =
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          module: "ESNext",
          moduleResolution: "bundler",
          skipLibCheck: true,
        },
        contentMappers: [{ package: "vize", extensions: [".vue"] }],
        include: ["*.vue"],
      },
      null,
      2,
    ) + "\n";
  const files = (row: (typeof source.cases)[number]) => ({ "tsconfig.json": config, ...row.files });
  const original = source.cases[0];
  assert.deepEqual(original.codes, []);
  return {
    typescript: "7.1.0-dev.20261017.1",
    vue: "3.5.20",
    baselineVize: "0.467.0",
    originals: [
      {
        issue: 9499,
        files: files(original),
        baselineCodes: [2344, 2344, 2344],
        baselineTerms: ["hasCloseBtn"],
        expectedCodes: [],
        expectedTerms: [],
        expectedStdout: "",
        expectedStderr: "",
        expectedStatus: 0,
      },
    ],
    controls: source.cases.slice(1).map((row) => ({
      name: `defaults-9499-${row.name}`,
      original: 9499,
      files: files(row),
      codes: [...row.codes],
      terms: row.codes.length ? ["not assignable"] : [],
      status: row.codes.length ? 2 : 0,
      stderr: "",
      ...(row.codes.length ? {} : { stdout: "" }),
    })),
  };
}

export function loadOfficialFixture(file: string, publishedSource?: string): Fixture {
  const value = JSON.parse(publishedSource ?? fs.readFileSync(file, "utf8"));
  const fixture = (
    publishedSource !== undefined && value.issue === 9499
      ? surplusDefaultsPublicFixture(value)
      : value
  ) as Fixture;
  assert.equal(fixture.typescript, "7.1.0-dev.20261017.1");
  assert.equal(fixture.vue, "3.5.20");
  assert.match(fixture.baselineVize, /^\d+\.\d+\.\d+$/u);
  assert.ok(Array.isArray(fixture.originals) && fixture.originals.length > 0);
  assert.ok(Array.isArray(fixture.controls));
  const issues = new Set(fixture.originals.map((row) => row.issue));
  assert.equal(issues.size, fixture.originals.length);
  for (const row of fixture.originals) {
    assert.ok(Number.isSafeInteger(row.issue) && row.issue > 0);
    for (const field of ["baselineCodes", "expectedCodes"] as const)
      assert.ok(Array.isArray(row[field]) && row[field].every(Number.isSafeInteger));
    for (const field of ["baselineTerms", "expectedTerms"] as const)
      assert.ok(
        Array.isArray(row[field]) && row[field].every((value) => typeof value === "string"),
      );
    for (const field of [
      "baselineStdout",
      "baselineStderr",
      "expectedStdout",
      "expectedStderr",
    ] as const)
      if (Object.hasOwn(row, field)) assert.equal(typeof row[field], "string", field);
    for (const field of ["baselineStatus", "expectedStatus"] as const)
      if (Object.hasOwn(row, field)) assert.ok(row[field] === 0 || row[field] === 2, field);
    if (row.baselineStatus !== undefined)
      assert.equal(row.baselineStatus, row.baselineCodes.length ? 2 : 0);
  }
  for (const row of officialCases(fixture)) {
    assert.match(row.name, /^[a-z0-9][a-z0-9-]*$/u);
    assert.ok(issues.has(row.original));
    assert.ok(Array.isArray(row.codes) && row.codes.every(Number.isSafeInteger));
    assert.ok(Object.hasOwn(row.files, "tsconfig.json"));
    for (const [name, content] of Object.entries(row.files)) {
      assert.ok(!path.isAbsolute(name) && !name.split(/[\\/]/u).includes(".."));
      assert.equal(typeof content, "string");
    }
    for (const field of ["stdout", "stderr"] as const)
      if (Object.hasOwn(row, field)) assert.equal(typeof row[field], "string", field);
    if (row.status !== undefined) assert.equal(row.status, row.codes.length ? 2 : 0);
  }
  return fixture;
}

export function officialCases(fixture: Fixture): Case[] {
  return [
    ...fixture.originals.map((row) => ({
      name: `issue-${row.issue}`,
      original: row.issue,
      files: structuredClone(row.files),
      codes: [...row.expectedCodes],
      terms: [...row.expectedTerms],
      ...(row.expectedStdout !== undefined ? { stdout: row.expectedStdout } : {}),
      ...(row.expectedStderr !== undefined ? { stderr: row.expectedStderr } : {}),
      ...(row.expectedStatus !== undefined ? { status: row.expectedStatus } : {}),
    })),
    ...structuredClone(fixture.controls),
  ];
}

export const officialDiagnostics = (stdout: string) =>
  [...stdout.matchAll(/^([^\r\n]+)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)].map((match) => ({
    file: match[1],
    line: Number(match[2]),
    column: Number(match[3]),
    code: Number(match[4]),
    message: match[5],
  }));

interface Result {
  signal: string | null;
  error: unknown;
  status: number | null;
  stdout: { bytes: number };
  stderr: { bytes: number };
  stdoutText: string;
  stderrText: string;
}

export function assertOfficialDiagnostics(
  row: Case,
  original: Original,
  baseline: boolean,
  result: Result,
  diagnostics = officialDiagnostics(result.stdoutText),
) {
  const codes = baseline ? original.baselineCodes : row.codes;
  const status = baseline ? original.baselineStatus : row.status;
  assert.equal(result.signal, null, row.name);
  assert.equal(result.error, null, row.name);
  assert.equal(result.status, status ?? (codes.length ? 2 : 0), row.name);
  assert.equal(result.stderr.bytes, 0, row.name);
  if (!baseline)
    assert.equal(
      [...result.stdoutText.matchAll(/\berror TS\d+:/gu)].length,
      diagnostics.length,
      `${row.name}: every TypeScript error must map to authored source`,
    );
  assert.deepEqual(
    diagnostics.map((diagnostic) => diagnostic.code).sort((left, right) => left - right),
    [...codes].sort((left, right) => left - right),
    row.name,
  );
  const terms = baseline ? original.baselineTerms : (row.terms ?? []);
  for (const term of terms)
    assert.ok(
      diagnostics.some((diagnostic) => diagnostic.message.includes(term)),
      `${row.name} must retain the original diagnostic term ${term}`,
    );
  const stdout = baseline ? original.baselineStdout : row.stdout;
  const stderr = baseline ? original.baselineStderr : row.stderr;
  if (stdout !== undefined) assert.equal(result.stdoutText, stdout, `${row.name}: whole stdout`);
  if (stderr !== undefined) assert.equal(result.stderrText, stderr, `${row.name}: whole stderr`);
  if (!codes.length) assert.equal(result.stdout.bytes, 0, row.name);
  if (row.build)
    assert.deepEqual(
      diagnostics
        .map((diagnostic) => diagnostic.file)
        .sort((left, right) => left.localeCompare(right)),
      ["strict-css/App.vue", "strict-props/App.vue"],
    );
  if (!baseline)
    for (const diagnostic of diagnostics) {
      const source = row.files[diagnostic.file];
      assert.ok(source !== undefined, `diagnostic must name authored file: ${row.name}`);
      const line = source.split(/\r?\n/u)[diagnostic.line - 1];
      assert.ok(
        line !== undefined && diagnostic.column >= 1 && diagnostic.column <= line.length + 1,
        `diagnostic must map into authored source: ${row.name}`,
      );
    }
  return diagnostics;
}

export async function verifyPublicInstallation(
  directory: string,
  output: string,
  observed: {
    packages: { name: string }[];
    platform: { name: string };
    mapper: { cli: { path: string } };
    nativeImage: { path: string };
    compiler: { path: string };
  },
) {
  const publication = JSON.parse(
    fs.readFileSync(path.join(output, "public-publication.json"), "utf8"),
  );
  const lock = JSON.parse(fs.readFileSync(path.join(directory, "package-lock.json"), "utf8"));
  type Locked = { version: string; resolved: string; integrity: string; optional?: boolean };
  const archives = [],
    absentOptional = [];
  for (const [relative, locked] of Object.entries(lock.packages as Record<string, Locked>)) {
    if (!relative) continue;
    const packageRoot = path.resolve(directory, relative);
    assert.ok(packageRoot.startsWith(path.resolve(directory) + path.sep));
    const manifestPath = path.join(packageRoot, "package.json");
    if (!fs.existsSync(manifestPath)) {
      assert.equal(locked.optional, true, "only optional foreign packages may be absent");
      absentOptional.push(relative);
      continue;
    }
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    assert.equal(manifest.version, locked.version);
    const archive = await readArchive(locked.resolved, locked.integrity);
    assert.equal(archive.archiveManifest.name, manifest.name);
    assert.equal(archive.archiveManifest.version, manifest.version);
    const released = publication.npm.find((item: { name: string }) => item.name === manifest.name);
    if (manifest.name === "vize" || manifest.name.startsWith("@vizejs/")) {
      assert.ok(released, "installed Vize package must belong to the verified publication");
      assert.equal(released.version, manifest.version);
      assert.equal(released.integrity, locked.integrity);
      assert.equal(released.sha256, archive.archive.sha256);
    }
    archives.push({
      directory: packageRoot,
      integrity: locked.integrity,
      tarball: locked.resolved,
      ...archive,
      ...observeInstalledArchive(packageRoot, archive.archiveEntries),
    });
  }
  for (const { name } of [...observed.packages, { name: observed.platform.name }])
    assert.ok(archives.some((item) => item.archiveManifest.name === name));
  for (const file of [observed.mapper.cli.path, observed.nativeImage.path, observed.compiler.path])
    assert.ok(
      archives.some((item) =>
        item.archiveEntries.some((entry) => path.join(item.directory, entry.path) === file),
      ),
      "actual executable must belong to a verified archive",
    );
  fs.writeFileSync(
    path.join(output, "installed-public-archives.json"),
    JSON.stringify({ archives, absentOptional }, null, 2) + "\n",
  );
}

export function resolveOfficialNativeImage(directory: string): string {
  const resolve = createRequire(
    path.join(directory, "node_modules/@vizejs/native/package.json"),
  ).resolve;
  const wrapper = JSON.parse(
    fs.readFileSync(path.join(directory, "node_modules/@vizejs/native/package.json"), "utf8"),
  );
  const candidates = Object.keys(wrapper.optionalDependencies).flatMap((name) => {
    try {
      return [resolve(`${name}/package.json`)];
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "MODULE_NOT_FOUND") return [];
      throw error;
    }
  });
  assert.equal(
    candidates.length,
    1,
    "public-only identity requires one unambiguous installed native package",
  );
  assert.ok(fs.realpathSync(candidates[0]).startsWith(fs.realpathSync(directory) + path.sep));
  const manifest = JSON.parse(fs.readFileSync(candidates[0], "utf8"));
  return fs.realpathSync(resolve(manifest.name));
}
