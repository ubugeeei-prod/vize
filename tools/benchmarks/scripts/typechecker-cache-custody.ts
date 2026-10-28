/** Untimed backend observer for the public authored benchmark only. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { prepareColdScale, verifyColdDiagnostics } from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { createColdScaleRuntime } from "./typechecker-cache-runtime.ts";
import {
  cachePathContains,
  inferCacheSemanticRoot,
  rootOutsideOwnedNamespace,
} from "./typechecker-cache-root.ts";
import { prepareCacheCarrier } from "./typechecker-cache-carrier.ts";
import { createColdBackendObserver } from "./typechecker-cache-backend.ts";
import { sealFixtureInputs } from "./typechecker-fixture-identity.ts";

export function summarizeColdBackendObserver(
  calls: string,
  ownedRoot: string | null,
  corpus?: {
    projectRoot: string;
    primaryRoot?: string;
    entries: { path: string }[];
  },
  metadataDirectory?: string,
  expectedSource?: string,
) {
  const root = ownedRoot ? realpathSync(ownedRoot) : null;
  const wrapper = resolve(calls, "..", "backend-observer.cjs");
  const wrapperSha256 = existsSync(wrapper)
    ? createHash("sha256").update(readFileSync(wrapper)).digest("hex")
    : null;
  const metadata =
    metadataDirectory && existsSync(metadataDirectory)
      ? readdirSync(metadataDirectory)
          .filter((name) => name.endsWith(".json"))
          .map((name) => JSON.parse(readFileSync(join(metadataDirectory, name), "utf8")))
      : [];
  const usedMetadata = new Set<number>();
  const rows = readdirSync(calls)
    .map((name) => {
      const directory = join(calls, name);
      const before = JSON.parse(readFileSync(join(directory, "before.json"), "utf8"));
      const after = JSON.parse(readFileSync(join(directory, "after.json"), "utf8"));
      const configPath = join(directory, "generated-tsconfig.json");
      const config =
        before.project && existsSync(configPath)
          ? JSON.parse(readFileSync(configPath, "utf8"))
          : null;
      const observedRoot =
        corpus && config
          ? inferCacheSemanticRoot({
              ...corpus,
              virtualRoot: before.physicalCwd,
              configPath: before.project,
              config,
            })
          : null;
      const explicitIndex = metadata.findIndex(
        (entry, index) =>
          !usedMetadata.has(index) &&
          entry.requestedConfigSha256 === before.configSha256 &&
          entry.requestedConfigPath === before.project &&
          JSON.stringify(entry.actualArgs) === JSON.stringify(before.argv),
      );
      const explicit = explicitIndex < 0 ? undefined : metadata[explicitIndex];
      if (explicitIndex >= 0) usedMetadata.add(explicitIndex);
      const alternatives = metadata.filter(
        (entry) =>
          entry.requestedConfigSha256 === before.configSha256 &&
          entry.requestedConfigPath === before.project &&
          JSON.stringify(entry.actualArgs) === JSON.stringify(before.argv),
      );
      const unambiguousMetadata = alternatives.every(
        (entry) =>
          entry.projectRoot === explicit?.projectRoot &&
          entry.virtualRoot === explicit?.virtualRoot,
      );
      const sourceRequirementSatisfied =
        !expectedSource ||
        (unambiguousMetadata &&
          explicit?.schema === "vize.cache-runtime-custody" &&
          explicit?.version === 1 &&
          explicit?.metadataOnly === true &&
          explicit?.compiledSourceRevision === expectedSource &&
          Array.isArray(explicit?.errors) &&
          explicit.errors.length === 0 &&
          explicit?.completion === "REAPED" &&
          explicit?.nativeSha256 === wrapperSha256 &&
          typeof explicit.nativeBinary === "string" &&
          resolve(explicit.nativeBinary) === wrapper &&
          typeof explicit.crateVersion === "string" &&
          /^\d+\.\d+\.\d+/.test(explicit.crateVersion) &&
          typeof explicit.virtualRoot === "string" &&
          resolve(explicit.virtualRoot) === before.physicalCwd &&
          typeof explicit.actualCwd === "string" &&
          resolve(explicit.actualCwd) === before.physicalCwd);
      const semanticRoot = expectedSource ? explicit?.projectRoot : observedRoot?.root;
      const rootQualified =
        !!semanticRoot &&
        sourceRequirementSatisfied &&
        observedRoot?.qualified === true &&
        (!expectedSource || observedRoot.root === semanticRoot) &&
        !!root &&
        rootOutsideOwnedNamespace(semanticRoot, root);
      return {
        directory: name,
        before,
        after,
        observedRoot,
        explicit,
        sourceRequirementSatisfied,
        sourceAttestation: {
          required: !!expectedSource,
          kind: expectedSource ? "compiled-observer-metadata" : "not-requested",
          actualAttested: !!expectedSource && sourceRequirementSatisfied,
        },
        rootAuthority: expectedSource
          ? "compiled-observer-metadata"
          : "inferred-primary-input-bijection",
        semanticRoot,
        rootQualified,
      };
    })
    .filter((row) => row.before.project !== null)
    .sort((a, b) => {
      const first = BigInt(a.before.startedMonotonicNs);
      const second = BigInt(b.before.startedMonotonicNs);
      return first < second ? -1 : first > second ? 1 : 0;
    });
  const distinct = new Map<string, (typeof rows)[number]>();
  const starts = new Set(rows.map((row) => row.before.startedMonotonicNs));
  const orderedUnambiguously = starts.size === rows.length;
  for (const row of rows) {
    const key = row.before.before?.physicalPath;
    // Cache namespaces must be absent on their first backend use. A later
    // child in the same Rust invocation may retain facts created by that run.
    if (key && !distinct.has(key)) distinct.set(key, row);
  }
  const initial = [...distinct.values()];
  const pathsInsideOwnedRoot =
    !!root &&
    rows.every(
      (row) =>
        row.before.before?.physicalPath &&
        cachePathContains(root, row.before.before.physicalPath) &&
        cachePathContains(root, row.before.physicalCwd),
    );
  const primaryBuildInfoAbsent =
    rows.length > 0 &&
    initial.length > 0 &&
    initial.every((row) => row.before.before.exists === false);
  const sourceMetadataComplete =
    !expectedSource || (metadata.length === rows.length && usedMetadata.size === rows.length);
  const semanticRootsOutsideCache = rows.length > 0 && rows.every((row) => row.rootQualified);
  const cacheMetadataCaptured = rows.every(
    (row) =>
      row.after.buildInfo?.exists === false ||
      (row.after.buildInfo?.exists === true && /^[a-f0-9]{64}$/.test(row.after.buildInfo.sha256)),
  );
  const completed = rows.every(
    (row) =>
      row.after.error === null && row.after.signal === null && [0, 1, 2].includes(row.after.status),
  );
  return {
    qualified:
      pathsInsideOwnedRoot &&
      primaryBuildInfoAbsent &&
      completed &&
      orderedUnambiguously &&
      cacheMetadataCaptured &&
      semanticRootsOutsideCache &&
      sourceMetadataComplete,
    orderedUnambiguously,
    cacheMetadataCaptured,
    semanticRootsOutsideCache,
    sourceMetadataComplete,
    pathsInsideOwnedRoot,
    primaryBuildInfoAbsentAtFirstBackendUse: primaryBuildInfoAbsent,
    completed,
    backendProjectInvocations: rows.length,
    distinctBuildInfoPaths: initial.length,
    rows,
  };
}

export function auditColdScaleCache(options: {
  provider: string;
  out: string;
  files: number;
  binary: string;
  carrierSfcs?: number;
  source?: string;
}) {
  assert.ok(
    process.platform !== "win32",
    "backend observer currently requires Unix executable scripts",
  );
  const { out, provider, files } = options;
  assert.ok(!existsSync(out), "refuse to overwrite custody evidence");
  mkdirSync(join(out, "tmp"), { recursive: true });
  const projectRoot = join(out, "project");
  const corpus = options.carrierSfcs
    ? prepareCacheCarrier(projectRoot, provider, options.carrierSfcs)
    : prepareColdScale(projectRoot, provider, files, false);
  const fixtureIdentity = sealFixtureInputs(projectRoot);
  const { command, backend } = coldScaleCommand({ ...options, tool: "cli", app: corpus.app });
  const runtime = createColdScaleRuntime(out);
  command.env = { ...command.env, ...runtime.env };
  const sha = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
  const executableSha256 = sha(command.executable),
    backendSha256 = sha(backend);
  try {
    const observer = createColdBackendObserver(backend, out);
    const args = [...command.args];
    args[args.indexOf("--corsa-path") + 1] = observer.wrapper;
    let sourceBuildIdentity: Record<string, unknown> | null = null;
    if (options.source) {
      assert.match(options.source, /^[a-f0-9]{40}$/u);
      const receipt = JSON.parse(
        readFileSync(command.executable + ".differential-build.json", "utf8"),
      );
      assert.equal(receipt.schema, "vize.differential.build");
      assert.equal(receipt.version, 1);
      assert.equal(receipt.recipe, "cargo build --release -p vize");
      assert.equal(receipt.sourceRevision, options.source);
      assert.equal(receipt.binarySha256, executableSha256);
      const version = spawnSync(command.executable, ["--version"], {
        encoding: "utf8",
        timeout: 30_000,
      });
      assert.equal(version.error, undefined);
      assert.equal(version.status, 0);
      assert.equal(version.stdout.trim(), receipt.cliVersion);
      sourceBuildIdentity = receipt;
    }
    const env: NodeJS.ProcessEnv = { ...process.env, ...command.env };
    delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_DIR;
    delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_METADATA_ONLY;
    delete env.VIZE_CACHE_CUSTODY_CAPTURE_DIR;
    const result = spawnSync(command.executable, args, {
      cwd: command.cwd,
      env,
      encoding: "utf8",
      timeout: 600_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    writeFileSync(join(out, "stdout.json"), result.stdout ?? "");
    writeFileSync(join(out, "stderr.txt"), result.stderr ?? "");
    assert.equal(result.status, 0, result.stderr);
    verifyColdDiagnostics(result.stdout ?? "", "cli", corpus.files, false);
    assert.equal(
      sealFixtureInputs(projectRoot).sha256,
      fixtureIdentity.sha256,
      "checker changed authored inputs",
    );
    const observed = summarizeColdBackendObserver(
      observer.calls,
      runtime.evidence.ownedRoot,
      { projectRoot, primaryRoot: corpus.app, entries: corpus.entries },
      undefined,
      undefined,
    );
    assert.equal(
      sha(command.executable),
      executableSha256,
      "checker replaced during untimed audit",
    );
    assert.equal(sha(backend), backendSha256, "backend replaced during untimed audit");
    const report = {
      schema: "vize.cold-cache-custody",
      version: 1,
      timingEligible: false,
      scope:
        "separate untimed public-corpus invocation; Node observer copies config and stats build-info before backend start",
      executableSha256,
      backendSha256,
      sourceBuildIdentity,
      semanticRootAuthority: "inferred-primary-input-bijection",
      directSourceRootAttested: false,
      corpusSha256: corpus.corpusSha256,
      providerRootPhysical: realpathSync(provider),
      fixtureIdentity,
      runtime: runtime.evidence,
      command: { ...command, args },
      ...observed,
    };
    writeFileSync(join(out, "corpus.json"), JSON.stringify(corpus, null, 2) + "\n");
    writeFileSync(join(out, "custody.json"), JSON.stringify(report, null, 2) + "\n");
    console.log(
      JSON.stringify({
        qualified: report.qualified,
        backendProjectInvocations: report.backendProjectInvocations,
        cacheBoundary: report.pathsInsideOwnedRoot,
        primaryBuildInfoAbsent: report.primaryBuildInfoAbsentAtFirstBackendUse,
      }),
    );
    return report;
  } finally {
    runtime.cleanup();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      files: { type: "string", default: "4323" },
      binary: { type: "string" },
      "carrier-sfcs": { type: "string" },
      source: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out && values.binary);
  const receipt = auditColdScaleCache({
    provider: resolve(values.provider),
    out: resolve(values.out),
    files: Number(values.files),
    binary: resolve(values.binary),
    carrierSfcs: values["carrier-sfcs"] ? Number(values["carrier-sfcs"]) : undefined,
    source: values.source,
  });
  assert.equal(
    receipt.qualified,
    true,
    "actual cache custody did not qualify; inspect retained evidence",
  );
}
