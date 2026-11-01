import assert from "node:assert/strict";
import { realpathSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { writeTarGz } from "../../tools/support/archive/tar.ts";
import { createHash } from "node:crypto";
import {
  cases,
  decodeInput,
  diagnostic,
  expected,
  input,
  profiles,
  sha256,
  sourcePins,
  sources,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/contract.ts";
import {
  assertProvenancePayload,
  gitBlob,
  validateCapsule,
  validateNative,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/authority.ts";
import {
  assertLocked,
  shippedImages,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/install.ts";
import {
  assessJson,
  expectedProgress,
  childEnvironment,
  healthy,
  recorder,
  retain,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/execute.ts";
import { argumentsFor } from "../../tools/benchmarks/scripts/public-typechecker-11317/run.ts";

const original = input();
const scratch = (action: (root: string) => void) => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11317-pure-")));
  try {
    action(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};

test("the five complete reporter files and settings remain pinned; only the role version changes", () => {
  assert.deepEqual(
    original.map((row) => [row.path, sha256(row.source)]),
    sourcePins,
  );
  assert.equal(original.length, 5);
  const before = sources(original, "before", "Original"),
    after = sources(original, "after", "Original");
  assert.deepEqual(before, original);
  assert.deepEqual(
    after.map((row) => ({
      ...row,
      source:
        row.path === "package.json"
          ? row.source.replace('"vize": "0.634.0"', '"vize": "0.632.0"')
          : row.source,
    })),
    original,
  );
  const packageJson = JSON.parse(before[0]!.source);
  assert.deepEqual(packageJson.devDependencies, {
    "typescript-native": "npm:typescript@7.0.2",
    vize: "0.632.0",
    vue: "3.5.44",
  });
  assert.deepEqual(packageJson.imports, { "#src/*": "./src/*.ts" });
  const config = JSON.parse(before[1]!.source);
  assert.deepEqual(config, {
    compilerOptions: {
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "bundler",
      strict: true,
      noEmit: true,
      skipLibCheck: true,
    },
    include: ["src/**/*.ts", "src/**/*.vue"],
  });
  const raw = readFileSync(
    fileURLToPath(
      new URL(
        "../_fixtures/differential/typechecker/package-import-redirects-11317/input.json",
        import.meta.url,
      ),
    ),
  );
  assert.throws(() => decodeInput(Buffer.concat([raw, Buffer.from("\n")])));
});

test("all nine ordered transforms preserve lines, restore the exact errors, and retain the complete oracle", () => {
  assert.deepEqual(cases, [
    "Original",
    "NoVersion",
    "Relative",
    "NoVue",
    "VueImporter",
    "Repair",
    "RestoreOriginal",
    "VueImporterRepair",
    "VueImporterRestore",
  ]);
  assert.deepEqual(
    sources(original, "after", "RestoreOriginal"),
    sources(original, "after", "Original"),
  );
  assert.deepEqual(
    sources(original, "after", "VueImporterRestore"),
    sources(original, "after", "VueImporter"),
  );
  assert.equal(
    sources(original, "after", "NoVersion")[0]!.source,
    sources(original, "after", "Original")[0]!.source.replace('  "version": "0.0.0",\n', ""),
  );
  assert.equal(
    sources(original, "after", "Relative")[2]!.source,
    'import { value } from "./plain";\nexport const v2 = value;\n',
  );
  assert.equal(
    sources(original, "after", "NoVue")[3]!.source.split("\n")[4],
    'const probe: number = "not a number";',
  );
  for (const id of cases) {
    const files = sources(original, "after", id),
      oracle = expected(original, id);
    assert.deepEqual(Object.keys(oracle), [
      "files",
      "programs",
      "errorCount",
      "warningCount",
      "fileCount",
    ]);
    assert.equal(
      files[3]!.source.split("\n")[4],
      id === "Repair" || id === "VueImporterRepair"
        ? "const probe: number = 1;"
        : 'const probe: number = "not a number";',
    );
    const rows = oracle.files as { file: string; diagnostics: string[] }[];
    assert.deepEqual(
      rows.find((row) => row.file === "src/plain.ts")!.diagnostics,
      oracle.errorCount ? [diagnostic] : [],
    );
    assert.deepEqual(
      rows.flatMap((row) => (row.file === "src/plain.ts" ? [] : row.diagnostics)),
      [],
    );
    assert.equal(oracle.fileCount, id.startsWith("VueImporter") ? 4 : 3);
  }
});

function capsule() {
  const profile = profiles.after;
  const receipts = ["linux-x64", "win32-x64", "darwin-x64", "darwin-arm64"].map(
    (targetPlatform) => {
      const raw = Buffer.from(
        JSON.stringify({
          schema: "vize.platformVsix.v1",
          version: profile.version,
          source: profile.candidate,
          provider: {
            source: profile.candidate,
            releaseVersion: profile.version,
            targetPlatform,
            rustTarget: "x86_64-unknown-linux-gnu",
            nodeFile: "native.node",
            nativeBuildToken: "token",
            files: [{ path: "native.node", bytes: 3, sha256: "abc" }],
          },
        }),
      );
      return {
        rawBase64: raw.toString("base64"),
        sha256: sha256(raw),
        origin: { runId: Number(profile.run), artifactId: 1, member: targetPlatform + ".json" },
      };
    },
  );
  const entry = {
    integrity: "sha512-" + Buffer.alloc(64, 1).toString("base64"),
    tarball: "https://registry.npmjs.org/vize/-/vize.tgz",
    metadataOrigin: "https://registry.npmjs.org/vize/0.634.0",
  };
  return {
    version: profile.version,
    tagSourceSha: profile.candidate,
    producerSource: profile.source,
    receipts,
    npmSRI: { vize: entry, "@vizejs/native": entry, "@vizejs/native-linux-x64-gnu": entry },
  };
}

test("role/source/run and unique four-provider custody are closed; image and token mutations refuse", () => {
  const good = capsule();
  validateCapsule("after", good);
  validateNative("after", good, { bytes: 3, sha256: "abc" }, "token");
  assert.throws(() => validateCapsule("before", good));
  for (const key of ["producerSource", "tagSourceSha", "version"] as const) {
    assert.throws(() => validateCapsule("after", { ...good, [key]: "wrong" }));
  }
  assert.throws(() =>
    validateCapsule("after", { ...good, receipts: [...good.receipts, good.receipts[0]] }),
  );
  assert.throws(() => validateCapsule("after", { ...good, receipts: good.receipts.slice(1) }));
  const foreign = structuredClone(good);
  foreign.receipts[0]!.origin.runId++;
  assert.throws(() => validateCapsule("after", foreign));
  const mutated = structuredClone(good);
  mutated.receipts[0]!.rawBase64 = Buffer.from("changed").toString("base64");
  assert.throws(() => validateCapsule("after", mutated));
  assert.throws(() => validateNative("after", good, { bytes: 4, sha256: "abc" }, "token"));
  assert.throws(() => validateNative("after", good, { bytes: 3, sha256: "wrong" }, "token"));
  assert.throws(() => validateNative("after", good, { bytes: 3, sha256: "abc" }, "foreign"));
  const lock = {
    packages: {
      "node_modules/vize": {
        version: "0.634.0",
        ...good.npmSRI.vize,
        resolved: good.npmSRI.vize.tarball,
      },
    },
  };
  assertLocked(lock, good, "vize", "0.634.0");
  assert.throws(() =>
    assertLocked(
      { packages: { "node_modules/vize": { ...lock.packages["node_modules/vize"], link: true } } },
      good,
      "vize",
      "0.634.0",
    ),
  );
  assert.equal(gitBlob(Buffer.from("hello\n")), "ce013625030ba8dba906f756967f9e9ca394464a");
});

test("whole-SRI archives authenticate every shipped file without rejecting npm-owned extra bins", () =>
  scratch((root) => {
    const manifest = Buffer.from('{"name":"example","version":"1.0.0"}\n'),
      body = Buffer.from("雪🌸\r\n");
    const payload = writeTarGz([
      { name: "package/package.json", data: manifest },
      { name: "package/index.js", data: body },
    ]);
    const integrity = "sha512-" + createHash("sha512").update(payload).digest("base64");
    writeFileSync(join(root, "package.json"), manifest);
    writeFileSync(join(root, "index.js"), body);
    mkdirSync(join(root, "node_modules/.bin"), { recursive: true });
    writeFileSync(join(root, "node_modules/.bin/npm-owned"), "not shipped by this archive");
    assert.equal(shippedImages(root, payload, integrity).images.length, 2);
    assert.throws(() =>
      shippedImages(root, payload, "sha512-" + Buffer.alloc(64).toString("base64")),
    );
    writeFileSync(join(root, "index.js"), "mutated");
    assert.throws(() => shippedImages(root, payload, integrity));
  }));

const packet = (
  root: string,
  label: string,
  stdout: Buffer | null,
  stderr: Buffer | null,
  status: number | null,
  error?: Error,
) =>
  retain(
    recorder(root),
    label,
    ["node", "npx-cli.js", "vize", "check", "-f", "json"],
    root,
    "start",
    { pid: 123, status, signal: null, stdout, stderr, error },
  );
test("complete raw/null/UTF8/status2 failure data exists before semantic guards and no JSON fields disappear", () =>
  scratch((root) => {
    const oracle = expected(original, "Original"),
      bytes = Buffer.from(JSON.stringify(oracle));
    const good = packet(root, "good", bytes, Buffer.from(expectedProgress(root, 3)), 1);
    assert.equal(assessJson(good, oracle, true).matchesAuthoredLaw, true);
    const extra = packet(
      root,
      "extra",
      Buffer.from(JSON.stringify({ ...oracle, unknownField: true })),
      Buffer.from(expectedProgress(root, 3)),
      1,
    );
    assert.throws(() => assessJson(extra, oracle, true));
    assert.equal(
      JSON.parse(readFileSync(join(root, "extra.process.json"), "utf8")).stdout,
      extra.stdout,
    );
    const bad = packet(root, "status2", bytes, Buffer.from("backend failed"), 2);
    assert.throws(() => assessJson(bad, oracle, true));
    assert.deepEqual(readFileSync(join(root, "status2.stderr.raw")), Buffer.from("backend failed"));
    const missing = packet(root, "spawn", null, null, null, new Error("ENOENT"));
    assert.equal(missing.stdoutImage, null);
    assert.equal(missing.stdout, null);
    assert.throws(() => healthy(missing, 0));
    assert.ok(JSON.parse(readFileSync(join(root, "spawn.process.json"), "utf8")).error);
    const invalid = packet(root, "utf8", Buffer.from([255]), Buffer.alloc(0), 1);
    assert.ok(invalid.stdoutUtf8Error);
    assert.deepEqual(readFileSync(join(root, "utf8.stdout.raw")), Buffer.from([255]));
    const debt = packet(
      root,
      "debt",
      Buffer.from(JSON.stringify(expected(original, "Original", 0))),
      Buffer.from(expectedProgress(root, 3)),
      0,
    );
    assert.equal(assessJson(debt, oracle, false).matchesAuthoredLaw, false);
    assert.throws(() => assessJson(debt, oracle, true));
  }));

test("no authority token reaches npm/native children and only exact arguments are accepted", () => {
  assert.deepEqual(
    childEnvironment({
      PATH: "/bin",
      GITHUB_TOKEN: "secret",
      GH_TOKEN: "secret",
      NODE_AUTH_TOKEN: "secret",
      ACTIONS_RUNTIME_TOKEN: "secret",
      password: "secret",
    }),
    { PATH: "/bin" },
  );
  assert.deepEqual(argumentsFor(["--out", "/a", "--npm", "/b", "--npx", "/c"]), {
    out: "/a",
    npm: "/b",
    npx: "/c",
  });
  assert.throws(() => argumentsFor(["--out", "relative", "--npm", "/b", "--npx", "/c"]));
  assert.throws(() => argumentsFor(["--out", "/a", "--npm", "/b", "--npm", "/c"]));
});

test("the new closed manual role preserves the old 0.602 job body and never selects it", () => {
  const workflow = readFileSync(
    fileURLToPath(
      new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    ),
    "utf8",
  ).replace(/\r\n/gu, "\n"); // YAML checkout EOL only; no process/diagnostic normalization.
  assert.ok(workflow.includes("default: legacy-0.602"));
  assert.ok(
    workflow.includes(
      "if: github.event_name != 'workflow_dispatch' || inputs.consumer == 'legacy-0.602'",
    ),
  );
  const oldJob = workflow
    .slice(workflow.indexOf("  public:\n"), workflow.indexOf("  contracts-11317:\n"))
    .replace(
      "    if: github.event_name != 'workflow_dispatch' || inputs.consumer == 'legacy-0.602'\n",
      "",
    );
  assert.equal(sha256(oldJob), "08e442156bc782895a717f5a3d7fa7ce77bd7bc965202713fd3eab3e246370b1");
  assert.ok(workflow.includes("inputs.consumer == 'package-import-redirects-11317'"));
  const controls = workflow.slice(
    workflow.indexOf("  contracts-11317:\n"),
    workflow.indexOf("  public-11317:\n"),
  );
  const publicRole = workflow.slice(workflow.indexOf("  public-11317:\n"));
  for (const job of [controls, publicRole]) {
    assert.ok(job.includes("node-version: 24.22.1"), "each job pins Node");
    assert.ok(job.includes("runs-on: blacksmith-8vcpu-ubuntu-2404"), "registered runner");
  }
  assert.ok(publicRole.includes("permissions: { contents: read, actions: read }"));
  assert.ok(publicRole.includes("needs: contracts-11317"));
  assert.ok(publicRole.includes("github.repository == 'ubugeeei-prod/vize'"));
  assert.ok(publicRole.includes("npm@11.19.1"));
  assert.ok(publicRole.includes("- uses: actions/upload-artifact@"));
  assert.ok(publicRole.includes("        if: always()"));
  assert.ok(!workflow.slice(0, workflow.indexOf("  public-11317:\n")).includes("actions: read"));
  assert.ok(!workflow.includes("CORSA_PATH"));
});

// Whole genuine public payload from failed 41323976400, not a fabricated success oracle.
const actual632Payload =
  "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoicGtnOm5wbS92aXplQDAuNjMyLjAiLCJkaWdlc3QiOnsic2hhNTEyIjoiNmVmOTUxYjFjNWVmMjJjZDdlZjI5OWIyMjcwNzIzMTRmZWQyYmViYWNkMTVkZjNjNjRiOWU0N2JhZjZkYWVhNjlhZDUwMWNhZTNiYjlhOGI5MTMyMmI0NmM1ZTcwZTU4MGFhMmFhNDA2NGY0OWYxNDVhMzc2MDU0M2Q3YTM1ZTEifX1dLCJwcmVkaWNhdGVUeXBlIjoiaHR0cHM6Ly9zbHNhLmRldi9wcm92ZW5hbmNlL3YxIiwicHJlZGljYXRlIjp7ImJ1aWxkRGVmaW5pdGlvbiI6eyJidWlsZFR5cGUiOiJodHRwczovL3Nsc2EtZnJhbWV3b3JrLmdpdGh1Yi5pby9naXRodWItYWN0aW9ucy1idWlsZHR5cGVzL3dvcmtmbG93L3YxIiwiZXh0ZXJuYWxQYXJhbWV0ZXJzIjp7IndvcmtmbG93Ijp7InJlZiI6InJlZnMvaGVhZHMvbWFpbiIsInJlcG9zaXRvcnkiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplIiwicGF0aCI6Ii5naXRodWIvd29ya2Zsb3dzL3JlbGVhc2UueW1sIn19LCJpbnRlcm5hbFBhcmFtZXRlcnMiOnsiZ2l0aHViIjp7ImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlcG9zaXRvcnlfaWQiOiIxMTIzNjEwMjE2IiwicmVwb3NpdG9yeV9vd25lcl9pZCI6IjI4ODgxMzM4MSJ9fSwicmVzb2x2ZWREZXBlbmRlbmNpZXMiOlt7InVyaSI6ImdpdCtodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplQHJlZnMvaGVhZHMvbWFpbiIsImRpZ2VzdCI6eyJnaXRDb21taXQiOiIxZWU3YjE0NDQ1OGFjMWJkMGMwNzE1NmNlMjQxODllNTljYTYwODBhIn19XX0sInJ1bkRldGFpbHMiOnsiYnVpbGRlciI6eyJpZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS9hY3Rpb25zL3J1bm5lci9naXRodWItaG9zdGVkIn0sIm1ldGFkYXRhIjp7Imludm9jYXRpb25JZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUvYWN0aW9ucy9ydW5zLzQxMzA1NjI1OTc0L2F0dGVtcHRzLzEifX19fQ==";
test("the retained real main/S/run/SRI payload passes; each foreign binding refuses", () => {
  const raw = Buffer.from(actual632Payload, "base64");
  assert.equal(sha256(raw), "81154657a27e10890733abb4d777897d237f238b45051ee7d8451665304a94a7");
  const payload = JSON.parse(raw.toString("utf8")) as {
      predicate: {
        buildDefinition: {
          externalParameters: { workflow: { ref: string; path: string; repository: string } };
          resolvedDependencies: { uri: string; digest: { gitCommit: string } }[];
        };
        runDetails: { metadata: { invocationId: string } };
      };
      subject: { name: string; digest: { sha512: string } }[];
    },
    integrity =
      "sha512-bvlRscXvIs1+8pmyJwcjFP7SvrrNFd88ZLnke69trqaa1QHK47uai5EyK0bF5w5YCqKqQGT0nxRaN2BUPXo14Q==";
  assert.equal(
    assertProvenancePayload("before", "vize", integrity, payload).source,
    profiles.before.source,
  );
  assert.throws(() => assertProvenancePayload("after", "vize", integrity, payload));
  const mutations = [
    (row: typeof payload) => {
      row.predicate.buildDefinition.externalParameters.workflow.ref = "refs/heads/release/v0.632.0";
    },
    (row: typeof payload) => {
      row.predicate.buildDefinition.externalParameters.workflow.path =
        ".github/workflows/foreign.yml";
    },
    (row: typeof payload) => {
      row.predicate.buildDefinition.externalParameters.workflow.repository =
        "https://github.com/foreign/repo";
    },
    (row: typeof payload) => {
      row.predicate.buildDefinition.resolvedDependencies[0].digest.gitCommit =
        profiles.before.candidate;
    },
    (row: typeof payload) => {
      row.predicate.buildDefinition.resolvedDependencies.push(
        structuredClone(row.predicate.buildDefinition.resolvedDependencies[0]),
      );
    },
    (row: typeof payload) => {
      row.predicate.runDetails.metadata.invocationId =
        "https://github.com/ubugeeei-prod/vize/actions/runs/1/attempts/1";
    },
    (row: typeof payload) => {
      row.predicate.runDetails.metadata.invocationId =
        "https://github.com/ubugeeei-prod/vize/actions/runs/41305625974/attempts/2";
    },
    (row: typeof payload) => {
      row.subject[0].name = "pkg:npm/foreign@0.632.0";
    },
    (row: typeof payload) => {
      row.subject[0].digest.sha512 = "0".repeat(128);
    },
  ];
  for (const mutate of mutations) {
    const row = structuredClone(payload);
    mutate(row);
    assert.throws(() => assertProvenancePayload("before", "vize", integrity, row));
  }
});

// Four genuine scoped publisher payloads: exact %40 URI spelling, no alias allowance.
const scopedPayloads = [
  {
    role: "before",
    name: "@vizejs/native",
    integrity:
      "sha512-uxm5/YDIgbG8Lo/5sGKz/75o8YZpwBsvVE8KciaKYzujxYKHd3KUHja7ILfhdIHrALoNLGBOexKbjJWg58AvrQ==",
    payload:
      "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoicGtnOm5wbS8lNDB2aXplanMvbmF0aXZlQDAuNjMyLjAiLCJkaWdlc3QiOnsic2hhNTEyIjoiYmIxOWI5ZmQ4MGM4ODFiMWJjMmU4ZmY5YjA2MmIzZmZiZTY4ZjE4NjY5YzAxYjJmNTQ0ZjBhNzIyNjhhNjMzYmEzYzU4Mjg3Nzc3Mjk0MWUzNmJiMjBiN2UxNzQ4MWViMDBiYTBkMmM2MDRlN2IxMjliOGM5NWEwZTdjMDJmYWQifX1dLCJwcmVkaWNhdGVUeXBlIjoiaHR0cHM6Ly9zbHNhLmRldi9wcm92ZW5hbmNlL3YxIiwicHJlZGljYXRlIjp7ImJ1aWxkRGVmaW5pdGlvbiI6eyJidWlsZFR5cGUiOiJodHRwczovL3Nsc2EtZnJhbWV3b3JrLmdpdGh1Yi5pby9naXRodWItYWN0aW9ucy1idWlsZHR5cGVzL3dvcmtmbG93L3YxIiwiZXh0ZXJuYWxQYXJhbWV0ZXJzIjp7IndvcmtmbG93Ijp7InJlZiI6InJlZnMvaGVhZHMvbWFpbiIsInJlcG9zaXRvcnkiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplIiwicGF0aCI6Ii5naXRodWIvd29ya2Zsb3dzL3JlbGVhc2UueW1sIn19LCJpbnRlcm5hbFBhcmFtZXRlcnMiOnsiZ2l0aHViIjp7ImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlcG9zaXRvcnlfaWQiOiIxMTIzNjEwMjE2IiwicmVwb3NpdG9yeV9vd25lcl9pZCI6IjI4ODgxMzM4MSJ9fSwicmVzb2x2ZWREZXBlbmRlbmNpZXMiOlt7InVyaSI6ImdpdCtodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplQHJlZnMvaGVhZHMvbWFpbiIsImRpZ2VzdCI6eyJnaXRDb21taXQiOiIxZWU3YjE0NDQ1OGFjMWJkMGMwNzE1NmNlMjQxODllNTljYTYwODBhIn19XX0sInJ1bkRldGFpbHMiOnsiYnVpbGRlciI6eyJpZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS9hY3Rpb25zL3J1bm5lci9naXRodWItaG9zdGVkIn0sIm1ldGFkYXRhIjp7Imludm9jYXRpb25JZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUvYWN0aW9ucy9ydW5zLzQxMzA1NjI1OTc0L2F0dGVtcHRzLzEifX19fQ==",
    sha256: "14bf08069016e463b0aac19def4ba95832484d18892f063af1994f40bf9f409f",
  },
  {
    role: "before",
    name: "@vizejs/native-linux-x64-gnu",
    integrity:
      "sha512-2OzF5C2UPJNsoVmFF5IiLAniiYqttDBq6vUtKonBcFnQP8ZTY9HS6wX8xMrPwDk9OvuzvD3XmbSOyrG7LRxQTQ==",
    payload:
      "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoicGtnOm5wbS8lNDB2aXplanMvbmF0aXZlLWxpbnV4LXg2NC1nbnVAMC42MzIuMCIsImRpZ2VzdCI6eyJzaGE1MTIiOiJkOGVjYzVlNDJkOTQzYzkzNmNhMTU5ODUxNzkyMjIyYzA5ZTI4OThhYWRiNDMwNmFlYWY1MmQyYTg5YzE3MDU5ZDAzZmM2NTM2M2QxZDJlYjA1ZmNjNGNhY2ZjMDM5M2QzYWZiYjNiYzNkZDc5OWI0OGVjYWIxYmIyZDFjNTA0ZCJ9fV0sInByZWRpY2F0ZVR5cGUiOiJodHRwczovL3Nsc2EuZGV2L3Byb3ZlbmFuY2UvdjEiLCJwcmVkaWNhdGUiOnsiYnVpbGREZWZpbml0aW9uIjp7ImJ1aWxkVHlwZSI6Imh0dHBzOi8vc2xzYS1mcmFtZXdvcmsuZ2l0aHViLmlvL2dpdGh1Yi1hY3Rpb25zLWJ1aWxkdHlwZXMvd29ya2Zsb3cvdjEiLCJleHRlcm5hbFBhcmFtZXRlcnMiOnsid29ya2Zsb3ciOnsicmVmIjoicmVmcy9oZWFkcy9tYWluIiwicmVwb3NpdG9yeSI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUiLCJwYXRoIjoiLmdpdGh1Yi93b3JrZmxvd3MvcmVsZWFzZS55bWwifX0sImludGVybmFsUGFyYW1ldGVycyI6eyJnaXRodWIiOnsiZXZlbnRfbmFtZSI6IndvcmtmbG93X2Rpc3BhdGNoIiwicmVwb3NpdG9yeV9pZCI6IjExMjM2MTAyMTYiLCJyZXBvc2l0b3J5X293bmVyX2lkIjoiMjg4ODEzMzgxIn19LCJyZXNvbHZlZERlcGVuZGVuY2llcyI6W3sidXJpIjoiZ2l0K2h0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemVAcmVmcy9oZWFkcy9tYWluIiwiZGlnZXN0Ijp7ImdpdENvbW1pdCI6IjFlZTdiMTQ0NDU4YWMxYmQwYzA3MTU2Y2UyNDE4OWU1OWNhNjA4MGEifX1dfSwicnVuRGV0YWlscyI6eyJidWlsZGVyIjp7ImlkIjoiaHR0cHM6Ly9naXRodWIuY29tL2FjdGlvbnMvcnVubmVyL2dpdGh1Yi1ob3N0ZWQifSwibWV0YWRhdGEiOnsiaW52b2NhdGlvbklkIjoiaHR0cHM6Ly9naXRodWIuY29tL3VidWdlZWVpLXByb2Qvdml6ZS9hY3Rpb25zL3J1bnMvNDEzMDU2MjU5NzQvYXR0ZW1wdHMvMSJ9fX19",
    sha256: "e41dd8a96d105e5abc35a1145817ce0a0979634dd59cf3acb785ebc240a366dd",
  },
  {
    role: "after",
    name: "@vizejs/native",
    integrity:
      "sha512-S1I0MYaBpZREXh+mKc+ZSykz2HAlcCut2GeEaSArWdVauqTWDYCsyBoRYUjNhTw+Jy/hREjRcGq/sZktUwbf4g==",
    payload:
      "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoicGtnOm5wbS8lNDB2aXplanMvbmF0aXZlQDAuNjM0LjAiLCJkaWdlc3QiOnsic2hhNTEyIjoiNGI1MjM0MzE4NjgxYTU5NDQ0NWUxZmE2MjljZjk5NGIyOTMzZDg3MDI1NzAyYmFkZDg2Nzg0NjkyMDJiNTlkNTVhYmFhNGQ2MGQ4MGFjYzgxYTExNjE0OGNkODUzYzNlMjcyZmUxNDQ0OGQxNzA2YWJmYjE5OTJkNTMwNmRmZTIifX1dLCJwcmVkaWNhdGVUeXBlIjoiaHR0cHM6Ly9zbHNhLmRldi9wcm92ZW5hbmNlL3YxIiwicHJlZGljYXRlIjp7ImJ1aWxkRGVmaW5pdGlvbiI6eyJidWlsZFR5cGUiOiJodHRwczovL3Nsc2EtZnJhbWV3b3JrLmdpdGh1Yi5pby9naXRodWItYWN0aW9ucy1idWlsZHR5cGVzL3dvcmtmbG93L3YxIiwiZXh0ZXJuYWxQYXJhbWV0ZXJzIjp7IndvcmtmbG93Ijp7InJlZiI6InJlZnMvaGVhZHMvbWFpbiIsInJlcG9zaXRvcnkiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplIiwicGF0aCI6Ii5naXRodWIvd29ya2Zsb3dzL3JlbGVhc2UueW1sIn19LCJpbnRlcm5hbFBhcmFtZXRlcnMiOnsiZ2l0aHViIjp7ImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlcG9zaXRvcnlfaWQiOiIxMTIzNjEwMjE2IiwicmVwb3NpdG9yeV9vd25lcl9pZCI6IjI4ODgxMzM4MSJ9fSwicmVzb2x2ZWREZXBlbmRlbmNpZXMiOlt7InVyaSI6ImdpdCtodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplQHJlZnMvaGVhZHMvbWFpbiIsImRpZ2VzdCI6eyJnaXRDb21taXQiOiI0M2NlN2QwODVmZGZlMjllMWZhNTNkZjUyNjhhYTY1ZGE2M2NkYWFjIn19XX0sInJ1bkRldGFpbHMiOnsiYnVpbGRlciI6eyJpZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS9hY3Rpb25zL3J1bm5lci9naXRodWItaG9zdGVkIn0sIm1ldGFkYXRhIjp7Imludm9jYXRpb25JZCI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUvYWN0aW9ucy9ydW5zLzQxMzE4NDU4Nzc5L2F0dGVtcHRzLzEifX19fQ==",
    sha256: "194d4b16ab9cd7e23be4eba356a6f08faa17fb6cfb5ec02c3911e5dd178f7421",
  },
  {
    role: "after",
    name: "@vizejs/native-linux-x64-gnu",
    integrity:
      "sha512-wQT4xad0IsFYT/HHNRAb3GCeS1wpJrMjJyQyS/hyk+CPyePf/HO87X9qq29yyWYKl7LLf7hvgTUltTcT7jUllQ==",
    payload:
      "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoicGtnOm5wbS8lNDB2aXplanMvbmF0aXZlLWxpbnV4LXg2NC1nbnVAMC42MzQuMCIsImRpZ2VzdCI6eyJzaGE1MTIiOiJjMTA0ZjhjNWE3NzQyMmMxNTg0ZmYxYzczNTEwMWJkYzYwOWU0YjVjMjkyNmIzMjMyNzI0MzI0YmY4NzI5M2UwOGZjOWUzZGZmYzczYmNlZDdmNmFhYjZmNzJjOTY2MGE5N2IyY2I3ZmI4NmY4MTM1MjViNTM3MTNlZTM1MjU5NSJ9fV0sInByZWRpY2F0ZVR5cGUiOiJodHRwczovL3Nsc2EuZGV2L3Byb3ZlbmFuY2UvdjEiLCJwcmVkaWNhdGUiOnsiYnVpbGREZWZpbml0aW9uIjp7ImJ1aWxkVHlwZSI6Imh0dHBzOi8vc2xzYS1mcmFtZXdvcmsuZ2l0aHViLmlvL2dpdGh1Yi1hY3Rpb25zLWJ1aWxkdHlwZXMvd29ya2Zsb3cvdjEiLCJleHRlcm5hbFBhcmFtZXRlcnMiOnsid29ya2Zsb3ciOnsicmVmIjoicmVmcy9oZWFkcy9tYWluIiwicmVwb3NpdG9yeSI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUiLCJwYXRoIjoiLmdpdGh1Yi93b3JrZmxvd3MvcmVsZWFzZS55bWwifX0sImludGVybmFsUGFyYW1ldGVycyI6eyJnaXRodWIiOnsiZXZlbnRfbmFtZSI6IndvcmtmbG93X2Rpc3BhdGNoIiwicmVwb3NpdG9yeV9pZCI6IjExMjM2MTAyMTYiLCJyZXBvc2l0b3J5X293bmVyX2lkIjoiMjg4ODEzMzgxIn19LCJyZXNvbHZlZERlcGVuZGVuY2llcyI6W3sidXJpIjoiZ2l0K2h0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemVAcmVmcy9oZWFkcy9tYWluIiwiZGlnZXN0Ijp7ImdpdENvbW1pdCI6IjQzY2U3ZDA4NWZkZmUyOWUxZmE1M2RmNTI2OGFhNjVkYTYzY2RhYWMifX1dfSwicnVuRGV0YWlscyI6eyJidWlsZGVyIjp7ImlkIjoiaHR0cHM6Ly9naXRodWIuY29tL2FjdGlvbnMvcnVubmVyL2dpdGh1Yi1ob3N0ZWQifSwibWV0YWRhdGEiOnsiaW52b2NhdGlvbklkIjoiaHR0cHM6Ly9naXRodWIuY29tL3VidWdlZWVpLXByb2Qvdml6ZS9hY3Rpb25zL3J1bnMvNDEzMTg0NTg3NzkvYXR0ZW1wdHMvMSJ9fX19",
    sha256: "40c0cc6a853b780e7954018ac8729369953fc7d25c54a18b7f83aefa22e2b45d",
  },
] as const;
test("all four real scoped payloads use the exact encoded namespace and source role", () => {
  for (const row of scopedPayloads) {
    const raw = Buffer.from(row.payload, "base64");
    assert.equal(sha256(raw), row.sha256);
    const value = JSON.parse(raw.toString("utf8")) as { subject: { name: string }[] };
    assertProvenancePayload(row.role, row.name, row.integrity, value);
    value.subject[0]!.name = "pkg:npm/" + row.name + "@" + profiles[row.role].version;
    assert.throws(() => assertProvenancePayload(row.role, row.name, row.integrity, value));
  }
});
