import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  createColdScaleRuntime,
  qualifyColdScaleCache,
} from "../../tools/benchmarks/scripts/typechecker-cache-runtime.ts";
import { createColdBackendObserver } from "../../tools/benchmarks/scripts/typechecker-cache-backend.ts";
import { summarizeColdBackendObserver } from "../../tools/benchmarks/scripts/typechecker-cache-custody.ts";
import { sealFixtureInputs } from "../../tools/benchmarks/scripts/typechecker-fixture-identity.ts";

const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");

void test("runtime cleanup removes only its marked UUID and refuses replacement", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cache-runtime-control-"));
  try {
    const out = path.join(root, "evidence"),
      base = path.join(root, "cache");
    fs.mkdirSync(out);
    fs.mkdirSync(base);
    fs.writeFileSync(path.join(base, "neighbour.txt"), "retain");
    const runtime = createColdScaleRuntime(out, base);
    assert.equal(runtime.evidence.createdEmpty, true);
    assert.ok(runtime.evidence.ownedRoot);
    const marker = path.join(runtime.evidence.ownedRoot, ".cold-scale-owner.json");
    const original = fs.readFileSync(marker);
    fs.writeFileSync(marker, "replaced");
    assert.throws(() => runtime.cleanup(), /ownership marker/);
    assert.equal(fs.readFileSync(path.join(base, "neighbour.txt"), "utf8"), "retain");
    fs.writeFileSync(marker, original);
    runtime.cleanup();
    assert.equal(fs.existsSync(runtime.evidence.ownedRoot), false);
    assert.equal(fs.readFileSync(path.join(base, "neighbour.txt"), "utf8"), "retain");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("actual backend observer preserves argv/bytes and distinguishes first absence, unknown and a later same-run fact", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cache-observer-control-"));
  const root = fs.realpathSync(temporary);
  try {
    const out = path.join(root, "evidence"),
      project = path.join(root, "source");
    fs.mkdirSync(out);
    fs.mkdirSync(path.join(project, "src"), { recursive: true });
    fs.writeFileSync(path.join(project, "src/Control.vue"), "<template>日本😀</template>\r\n");
    const runtime = createColdScaleRuntime(out, path.join(root, "cache"));
    const namespace = path.join(runtime.evidence.ownedRoot!, "namespace");
    fs.mkdirSync(path.join(namespace, "src"), { recursive: true });
    fs.writeFileSync(path.join(namespace, "src/Control.vue.ts"), "export {};\n");
    const config = path.join(namespace, "tsconfig.json");
    fs.writeFileSync(
      config,
      JSON.stringify({
        compilerOptions: { incremental: false },
        files: [path.join(namespace, "src/Control.vue.ts")],
      }),
    );
    const backend = path.join(root, "backend.cjs");
    // Protocol stubs validate custody laws only; they are not Vize/hardware evidence.
    fs.writeFileSync(
      backend,
      `#!${process.execPath}
const fs=require("node:fs"),a=process.argv.slice(2);
fs.writeFileSync(a[a.indexOf("--tsBuildInfoFile")+1],"current-run build info");
process.stdout.write("native stdout 日本\\n"); process.stderr.write("native stderr\\n");
process.exit(2);
`,
      { mode: 0o755 },
    );
    const observer = createColdBackendObserver(backend, out);
    const buildInfo = path.join(namespace, "program.tsbuildinfo");
    const argv = ["--project", config, "--incremental", "--tsBuildInfoFile", buildInfo];
    for (let index = 0; index < 2; index++) {
      const result = spawnSync(observer.wrapper, argv, { cwd: namespace, encoding: "utf8" });
      assert.equal(result.status, 2);
      assert.equal(result.stdout, "native stdout 日本\n");
      assert.equal(result.stderr, "native stderr\n");
    }
    const corpus = { projectRoot: project, entries: [{ path: "src/Control.vue" }] };
    const accepted = summarizeColdBackendObserver(
      observer.calls,
      runtime.evidence.ownedRoot,
      corpus,
    );
    assert.equal(accepted.qualified, true);
    assert.equal(accepted.backendProjectInvocations, 2);
    assert.equal(accepted.distinctBuildInfoPaths, 1);
    assert.deepEqual(accepted.rows[0].before.argv, argv);
    assert.equal(accepted.rows[0].before.before.exists, false);
    assert.equal(accepted.rows[1].before.before.exists, true);
    assert.equal(accepted.rows[0].after.buildInfo.sha256, hash(buildInfo));
    assert.equal(accepted.rows[0].before.effectiveIncremental, true);
    assert.equal(accepted.rows[0].before.configuredIncremental, false);
    const first = path.join(observer.calls, accepted.rows[0].directory, "before.json");
    const firstBytes = fs.readFileSync(first, "utf8");
    const unknown = JSON.parse(firstBytes);
    unknown.before.exists = null;
    unknown.before.error = "EACCES";
    fs.writeFileSync(first, JSON.stringify(unknown));
    assert.equal(
      summarizeColdBackendObserver(observer.calls, runtime.evidence.ownedRoot, corpus).qualified,
      false,
    );
    fs.writeFileSync(first, firstBytes);
    const metadata = path.join(out, "source-root");
    fs.mkdirSync(metadata);
    const source = "a".repeat(40);
    const entry = {
      schema: "vize.cache-runtime-custody",
      version: 1,
      metadataOnly: true,
      projectRoot: project,
      virtualRoot: namespace,
      requestedConfigPath: config,
      requestedConfigSha256: hash(config),
      actualArgs: argv,
      actualCwd: namespace,
      nativeBinary: observer.wrapper,
      nativeSha256: hash(observer.wrapper),
      crateVersion: "0.474.0",
      compiledSourceRevision: source,
      errors: [],
      completion: "REAPED",
    };
    for (const name of ["first.json", "second.json"])
      fs.writeFileSync(path.join(metadata, name), JSON.stringify(entry));
    assert.equal(
      summarizeColdBackendObserver(
        observer.calls,
        runtime.evidence.ownedRoot,
        corpus,
        metadata,
        source,
      ).qualified,
      true,
    );
    entry.compiledSourceRevision = "b".repeat(40);
    for (const name of ["first.json", "second.json"])
      fs.writeFileSync(path.join(metadata, name), JSON.stringify(entry));
    assert.equal(
      summarizeColdBackendObserver(
        observer.calls,
        runtime.evidence.ownedRoot,
        corpus,
        metadata,
        source,
      ).qualified,
      false,
    );
    for (const invalidRoot of [namespace, runtime.evidence.ownedRoot!, root]) {
      entry.compiledSourceRevision = source;
      entry.projectRoot = invalidRoot;
      for (const name of ["first.json", "second.json"])
        fs.writeFileSync(path.join(metadata, name), JSON.stringify(entry));
      assert.equal(
        summarizeColdBackendObserver(
          observer.calls,
          runtime.evidence.ownedRoot,
          corpus,
          metadata,
          source,
        ).qualified,
        false,
        "malformed source roots disagree with the observed primary root",
      );
    }
    fs.writeFileSync(
      path.join(namespace, "src/Control.vue"),
      fs.readFileSync(path.join(project, "src/Control.vue")),
    );
    entry.projectRoot = namespace;
    entry.compiledSourceRevision = source;
    for (const name of ["first.json", "second.json"])
      fs.writeFileSync(path.join(metadata, name), JSON.stringify(entry));
    const nested = summarizeColdBackendObserver(
      observer.calls,
      runtime.evidence.ownedRoot,
      { projectRoot: namespace, entries: [{ path: "src/Control.vue" }] },
      metadata,
      source,
    );
    assert.equal(nested.rows[0].observedRoot?.qualified, true);
    assert.equal(nested.rows[0].observedRoot?.root, namespace);
    assert.equal(nested.rows[0].explicit.projectRoot, namespace);
    assert.equal(
      nested.qualified,
      false,
      "matching observed and attested roots nested inside cache must fail specifically at containment",
    );
    runtime.cleanup();
    assert.equal(fs.existsSync(project), true);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

void test("a custody receipt cannot migrate to another executable, provider, corpus or cache context", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cache-identity-control-"));
  try {
    const provider = path.join(root, "provider"),
      out = path.join(root, "evidence");
    fs.mkdirSync(provider);
    fs.mkdirSync(out);
    const runtime = createColdScaleRuntime(out, path.join(root, "cache"));
    const receipt = path.join(out, "custody.json");
    const identity = {
      schema: "vize.cold-cache-custody",
      version: 1,
      qualified: true,
      executableSha256: "a".repeat(64),
      backendSha256: "b".repeat(64),
      corpusSha256: "c".repeat(64),
      providerRootPhysical: fs.realpathSync(provider),
      runtime: runtime.evidence,
      fixtureIdentity: { sha256: "e".repeat(64) },
    };
    fs.writeFileSync(receipt, JSON.stringify(identity));
    const request = {
      receiptPath: receipt,
      provider,
      runtime: runtime.evidence,
      executableSha256: identity.executableSha256,
      backendSha256: identity.backendSha256,
      corpusSha256: identity.corpusSha256,
      fixtureSha256: "e".repeat(64),
    };
    assert.equal(qualifyColdScaleCache(request).qualified, true);
    for (const field of ["executableSha256", "backendSha256", "corpusSha256"] as const) {
      assert.equal(qualifyColdScaleCache({ ...request, [field]: "d".repeat(64) }).qualified, false);
    }
    assert.equal(
      qualifyColdScaleCache({
        ...request,
        runtime: {
          ...runtime.evidence,
          context: { ...runtime.evidence.context, outputParentPhysical: "/different-context" },
        },
      }).qualified,
      false,
    );
    runtime.cleanup();
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("authored compiler-option changes invalidate the seal without traversing dependency loops", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-seal-control-"));
  try {
    fs.writeFileSync(path.join(root, "App.vue"), "<template/>");
    fs.writeFileSync(path.join(root, "tsconfig.json"), '{"compilerOptions":{"strict":true}}');
    fs.symlinkSync(root, path.join(root, "node_modules"), "dir");
    const before = sealFixtureInputs(root);
    fs.writeFileSync(
      path.join(root, "tsconfig.json"),
      '{"compilerOptions":{"strict":false,"noCheck":true}}',
    );
    const after = sealFixtureInputs(root);
    assert.equal(before.files, 2);
    assert.equal(after.files, 2);
    assert.notEqual(
      before.sha256,
      after.sha256,
      "changing check coverage cannot reuse an old oracle/custody receipt",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
