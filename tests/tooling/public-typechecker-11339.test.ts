import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  originalFiles,
  wrappedSupplement,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/original.ts";
import {
  input,
  sources,
  sha256,
  originalFacts,
  profileFor,
  beforeProfile,
  afterProfile,
  limitations,
  type Case,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/contract.ts";
import {
  validateCapsule,
  validateNative,
  assertProvenancePayload,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/authority.ts";
import {
  recorder,
  guardedCapture,
  assessFacts,
  commandsFor,
  inputImages,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/execute.ts";
import { retain } from "../../tools/benchmarks/scripts/public-typechecker-11317/execute.ts";
import { fileImage } from "../../tools/support/release/computed_event_custody.ts";
import { sourceBin } from "../../tools/benchmarks/scripts/public-typechecker-11339/install.ts";
import {
  verifyAllBeforeAssessment,
  argumentsFor,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/run.ts";
const scratch = (action: (root: string) => void) => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11339-pure-")));
  try {
    action(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};

test("exact three source bodies remain separate from authored package/config and Wrapped supplement", () => {
  assert.deepEqual(
    originalFiles.map((file) => [file.path, Buffer.byteLength(file.source), sha256(file.source)]),
    [
      ["src/shared.ts", 299, "4682d621fbfdfd9ceb1d93e422ab6d83b54a927f4e3c0a2621724947afbccfe3"],
      ["src/base.ts", 146, "5f5d1cad14e80f58224306173db52efbebd4c12498d753c80ea328a3f9ad2f9b"],
      ["src/Invalid.vue", 350, "c0094aa36df53290e783e5b00d37dd1c3a04a92ced5dce7295a64e43ff5e3f1e"],
    ],
  );
  assert.equal(
    sha256(wrappedSupplement),
    "0a48d6f8366ba8fd8acc4cdf06cd93727ae1ed68629a5cce395d5dc1eab986a6",
  );
  for (const vue of ["3.4.23", "3.6.0-beta.10"] as const) {
    const files = input(vue),
      pkg = JSON.parse(files[0]!.source),
      config = JSON.parse(files[1]!.source);
    assert.deepEqual(pkg.devDependencies, {
      vize: "0.635.0",
      vue,
      typescript: "6.0.3",
      "vue-tsc": "3.0.8",
    });
    assert.deepEqual(config, {
      compilerOptions: {
        strict: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
      },
      include: ["src/**/*"],
    });
    assert.equal(files.length, 5);
    assert.equal(originalFiles.length, 3);
    assert.equal(limitations.exactReportedCompilerClaimed, false);
    assert.equal(limitations.selectedCorsaBuild, "UNKNOWN");
  }
});
test("valid repair changes only the authored template line; restore retains every original byte", () => {
  const files = input("3.4.23"),
    repair = sources(files, "before", "Repair");
  for (const file of files)
    if (file.path !== "src/Invalid.vue")
      assert.equal(repair.find((row) => row.path === file.path)!.source, file.source);
  assert(repair.find((file) => file.path === "src/Invalid.vue")!.source.includes("greet('x')"));
  assert(!repair.find((file) => file.path === "src/Invalid.vue")!.source.includes("{{ missing }}"));
  assert.deepEqual(
    sources(files, "before", "RestoreOriginal"),
    sources(files, "before", "Original"),
  );
  const supplemental = sources(files, "before", "WrappedSupplement");
  assert.deepEqual(supplemental.slice(0, -1), files);
  assert.equal(supplemental.at(-1)!.path, "src/Wrapped.vue");
});
test("the unknown containing release refuses before any public operation", () => {
  assert.equal(afterProfile, null);
  assert.throws(() => profileFor("after"), /UNKNOWN/u);
  assert.equal(profileFor("before").capsule, "9b9242d60d7121df1b8454697d1eec37e2d16309");
  assert.throws(() => argumentsFor(["--out", "relative", "--npm", "/b", "--npx", "/c"]));
  assert.throws(() => argumentsFor(["--out", "/a", "--npm", "/b", "--npm", "/c"]));
});
test("default installed Vize retains literal check argv without Options API or provider overrides", () => {
  assert.deepEqual(commandsFor("/project", "/manager/npx-cli.js"), [
    { kind: "default-cli-text", argv: ["/project/node_modules/.bin/vize", "check"] },
    {
      kind: "authored-cli-json-supplement",
      argv: ["/project/node_modules/.bin/vize", "check", "-f", "json"],
    },
    {
      kind: "reported-independent-reference",
      argv: [process.execPath, "/manager/npx-cli.js", "vue-tsc", "--noEmit"],
    },
  ]);
});
function mockCapsule() {
  const p = beforeProfile,
    integrity = "sha512-" + Buffer.alloc(64, 1).toString("base64");
  return {
    version: p.version,
    tagSourceSha: p.candidate,
    producerSource: p.source,
    receipts: ["darwin-arm64", "darwin-x64", "linux-x64", "win32-x64"].map((targetPlatform) => {
      const raw = Buffer.from(
        JSON.stringify({
          schema: "vize.platformVsix.v1",
          version: p.version,
          source: p.candidate,
          provider: {
            source: p.candidate,
            releaseVersion: p.version,
            targetPlatform,
            rustTarget: "x86_64-unknown-linux-gnu",
            nodeFile: "native.node",
            files: [{ path: "native.node", bytes: 3, sha256: "a".repeat(64) }],
            nativeBuildToken: "b".repeat(32),
          },
        }),
      );
      return {
        rawBase64: raw.toString("base64"),
        sha256: sha256(raw),
        origin: { runId: Number(p.run), artifactId: 1, member: targetPlatform + ".json" },
      };
    }),
    npmSRI: Object.fromEntries(
      ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"].map((name) => [
        name,
        {
          integrity,
          tarball: "https://registry.npmjs.org/pkg/-/pkg.tgz",
          metadataOrigin: "https://registry.npmjs.org/pkg/0.635.0",
        },
      ]),
    ),
  };
}
test("synthetic authority controls refuse wrong source, image, role and duplicate providers", () => {
  const good = mockCapsule();
  validateCapsule("before", good);
  validateNative("before", good, { bytes: 3, sha256: "a".repeat(64) }, "b".repeat(32));
  assert.throws(() => validateCapsule("after", good));
  assert.throws(() =>
    validateCapsule("before", { ...good, producerSource: beforeProfile.candidate }),
  );
  assert.throws(() =>
    validateCapsule("before", { ...good, receipts: [...good.receipts, good.receipts[0]] }),
  );
  assert.throws(() =>
    validateNative("before", good, { bytes: 4, sha256: "a".repeat(64) }, "b".repeat(32)),
  );
  assert.throws(() =>
    validateNative("before", good, { bytes: 3, sha256: "a".repeat(64) }, "c".repeat(32)),
  );
});
test("role-owned main/S/run/SRI and exact encoded scoped PURL remain closed", () => {
  const integrity = "sha512-" + Buffer.alloc(64, 1).toString("base64"),
    p = beforeProfile;
  const payload = {
    _type: "https://in-toto.io/Statement/v1",
    predicateType: "https://slsa.dev/provenance/v1",
    subject: [
      {
        name: "pkg:npm/%40vizejs/native@0.635.0",
        digest: { sha512: Buffer.alloc(64, 1).toString("hex") },
      },
    ],
    predicate: {
      buildDefinition: {
        buildType: "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1",
        externalParameters: {
          workflow: {
            ref: "refs/heads/main",
            repository: "https://github.com/ubugeeei-prod/vize",
            path: ".github/workflows/release.yml",
          },
        },
        resolvedDependencies: [
          {
            uri: "git+https://github.com/ubugeeei-prod/vize@refs/heads/main",
            digest: { gitCommit: p.source },
          },
        ],
      },
      runDetails: {
        metadata: {
          invocationId:
            "https://github.com/ubugeeei-prod/vize/actions/runs/" + p.run + "/attempts/1",
        },
      },
    },
  };
  assertProvenancePayload("before", "@vizejs/native", integrity, payload);
  const wrong = structuredClone(payload);
  wrong.subject[0]!.name = "pkg:npm/@vizejs/native@0.635.0";
  assert.throws(() => assertProvenancePayload("before", "@vizejs/native", integrity, wrong));
  const candidate = structuredClone(payload);
  candidate.predicate.buildDefinition.resolvedDependencies[0]!.digest.gitCommit = p.candidate;
  assert.throws(() => assertProvenancePayload("before", "@vizejs/native", integrity, candidate));
});
function mockDto(id: Case) {
  const files = sources(input("3.4.23"), "before", id),
    paths = files
      .filter((file) => file.path.startsWith("src/"))
      .map((file) => file.path)
      .sort();
  const facts = id === "Repair" ? [] : [...originalFacts];
  const rows = paths.map((file) => ({
    file,
    diagnostics: facts
      .filter((fact) => fact.file === file)
      .map(
        (fact) =>
          `error:${fact.line}:${fact.column} [${fact.code}] ${fact.message ?? "Property 'missing' does not exist on type 'MOCK_UNOWNED_RECEIVER'."}`,
      ),
  }));
  return {
    files,
    dto: {
      files: rows,
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions: JSON.parse(files[1]!.source).compilerOptions,
          files: paths,
        },
      ],
      errorCount: facts.length,
      warningCount: 0,
      fileCount: paths.length,
    },
  };
}
test("whole mock DTO fields and five facts are checked without inventing the missing generic receiver", () =>
  scratch((root) => {
    const record = recorder(root),
      f = mockDto("Original");
    const packet = retain(record, "mock", ["vize", "check", "-f", "json"], root, "start", {
      pid: 123,
      status: 1,
      signal: null,
      stdout: Buffer.from(JSON.stringify(f.dto)),
      stderr: Buffer.from("exact unowned progress\r\n"),
    });
    const assessed = assessFacts("after", "Original", packet, f.files);
    assert.equal(assessed.publicAcceptance, false);
    assert.deepEqual(
      readFileSync(join(root, "mock.stderr.raw")),
      Buffer.from("exact unowned progress\r\n"),
    );
    for (const mutate of [
      (dto: typeof f.dto) => {
        dto.files.pop();
      },
      (dto: typeof f.dto) => {
        dto.files[0]!.diagnostics.pop();
        dto.errorCount--;
      },
      (dto: typeof f.dto) => {
        dto.fileCount = 0;
      },
      (dto: typeof f.dto) => {
        dto.programs[0]!.files.reverse();
      },
    ]) {
      const dto = structuredClone(f.dto);
      mutate(dto);
      assert.throws(() =>
        assessFacts("after", "Original", { ...packet, stdout: JSON.stringify(dto) }, f.files),
      );
    }
    assert.throws(() =>
      assessFacts(
        "after",
        "Original",
        { ...packet, stdout: JSON.stringify({ ...f.dto, extra: true }) },
        f.files,
      ),
    );
    const repaired = mockDto("Repair");
    assert.equal(
      assessFacts(
        "after",
        "Repair",
        { ...packet, status: 0, stdout: JSON.stringify(repaired.dto) },
        repaired.files,
      ).publicAcceptance,
      false,
    );
  }));
test("unknown bin route refuses before callback and retains a complete non-executed attempt", () =>
  scratch((root) => {
    const directory = join(root, "node_modules/vize");
    mkdirSync(join(directory, "bin"), { recursive: true });
    mkdirSync(join(root, "node_modules/.bin"), { recursive: true });
    writeFileSync(join(directory, "package.json"), JSON.stringify({ bin: { vize: "bin/vize" } }));
    writeFileSync(join(directory, "bin/vize"), "inert authenticated public fixture");
    const image = fileImage(join(directory, "bin/vize"));
    symlinkSync("../vize/bin/vize", join(root, "node_modules/.bin/vize"));
    sourceBin(root, "vize", [image]);
    rmSync(join(root, "node_modules/.bin/vize"));
    writeFileSync(join(root, "node_modules/.bin/vize"), "unknown shim");
    let executions = 0;
    const packet = guardedCapture(
      recorder(root),
      "refused",
      [join(root, "node_modules/.bin/vize"), "check"],
      root,
      () => {
        sourceBin(root, "vize", [image]);
      },
      () => {
        executions++;
        throw Error("must not run");
      },
    );
    assert.equal(executions, 0);
    assert.equal(packet.status, null);
    assert(packet.error);
    assert.equal(packet.stdout, null);
  }));
test("changed or extra authored source refuses before each command callback", () =>
  scratch((root) => {
    const files = input("3.4.23");
    mkdirSync(join(root, "src"));
    for (const file of files) writeFileSync(join(root, file.path), file.source);
    inputImages(root, files);
    const changed = files.find((file) => file.path === "src/Invalid.vue")!;
    for (const mutate of [
      () => writeFileSync(join(root, changed.path), changed.source + "\n"),
      () => writeFileSync(join(root, "src/foreign.ts"), "foreign"),
      () => rmSync(join(root, changed.path)),
    ]) {
      writeFileSync(join(root, changed.path), changed.source);
      rmSync(join(root, "src/foreign.ts"), { force: true });
      mutate();
      let executed = 0;
      const packet = guardedCapture(
        recorder(root),
        "input-refusal",
        commandsFor(root, "/manager/npx")[0]!.argv,
        root,
        () => {
          inputImages(root, files);
        },
        () => {
          executed++;
          throw Error("must not execute");
        },
      );
      assert.equal(executed, 0);
      assert.equal(packet.status, null);
      assert.equal(packet.stdout, null);
      assert(packet.error);
    }
  }));
test("every final role custody completes before semantics; an early failure cannot skip another custody", () => {
  const order: string[] = [];
  const errors = verifyAllBeforeAssessment([
    {
      verify() {
        order.push("before");
        throw Error("changed");
      },
    },
    {
      verify() {
        order.push("after");
      },
    },
  ]);
  assert.deepEqual(order, ["before", "after"]);
  assert.notEqual(errors[0], null);
  assert.equal(errors[1], null);
});
test("new closed choice preserves old602 and11317 workflow bodies byte-for-byte", () => {
  const workflow = readFileSync(
    fileURLToPath(
      new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    ),
    "utf8",
  ).replace(/\r\n/gu, "\n");
  const old = workflow
    .slice(workflow.indexOf("  public:\n"), workflow.indexOf("  contracts-11317:\n"))
    .replace(
      "    if: github.event_name != 'workflow_dispatch' || inputs.consumer == 'legacy-0.602'\n",
      "",
    );
  assert.equal(sha256(old), "08e442156bc782895a717f5a3d7fa7ce77bd7bc965202713fd3eab3e246370b1");
  const inherited = workflow.slice(
    workflow.indexOf("  contracts-11317:\n"),
    workflow.indexOf("  contracts-11339:\n"),
  );
  assert.equal(
    sha256(inherited),
    "9fd32640616e647056ab120fcd4909348dcf6734d77839c417aac3d5a66a3000",
  );
  assert(workflow.includes("options-inherited-members-11339"));
  assert(workflow.includes("needs: contracts-11339"));
});
