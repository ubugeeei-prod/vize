import { isDeepStrictEqual } from "node:util";
import { retainFinalCustody } from "../../tools/benchmarks/scripts/public-typechecker-11317/run.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { assertCustody, fileImage } from "../../tools/support/release/computed_event_custody.ts";
import { nativeLoaderProgram } from "../../tools/support/release/human_reports_11252_11253_public/native-loader-program.ts";
import { probeAfterNativePrerequisite } from "../../tools/support/release/human_reports_11252_11253_public/native-loader-prerequisite.ts";
import type { PackageBase } from "../../tools/support/release/human_reports_11252_11253_public/model.ts";
import {
  assertManager,
  managerImages,
  projectBin,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/install.ts";
import {
  cases,
  expected,
  input,
  profiles,
  sha256,
  sources,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/contract.ts";
import {
  assess,
  assessCaptured,
  assessJson,
  expectedProgress,
  recorder,
  retain,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/execute.ts";

const assertionData = (actual: unknown, expected: unknown) => (error: unknown) =>
  error instanceof assert.AssertionError &&
  isDeepStrictEqual(error.actual, actual) &&
  isDeepStrictEqual(error.expected, expected);

const scratch = (action: (root: string) => void) => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11317-route-")));
  try {
    action(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};
const write = (file: string, data: string | Buffer) => {
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, data);
};

test("npm/npx declared entries and the actual default vize link refuse foreign routes", () =>
  scratch((root) => {
    const npm = join(root, "manager/bin/npm-cli.js"),
      npx = join(root, "manager/bin/npx-cli.js");
    write(npm, "npm");
    write(npx, "npx");
    write(
      join(root, "manager/package.json"),
      JSON.stringify({
        name: "npm",
        version: "11.19.1",
        bin: { npm: "bin/npm-cli.js", npx: "bin/npx-cli.js" },
      }),
    );
    const manager = managerImages(npm, npx);
    assert.equal(manager.length, 4);
    write(npx, "changed-after-install");
    let executes = 0;
    assert.throws(
      () => {
        assertManager(manager, npm, npx);
        executes++;
      },
      assertionData(
        fileImage(npx),
        manager.find((image) => image.path === npx),
      ),
    );
    assert.equal(executes, 0);
    write(npx, "npx");
    write(join(root, "foreign-npx.js"), "foreign");
    assert.throws(
      () => managerImages(npm, join(root, "foreign-npx.js")),
      assertionData(realpathSync(join(root, "foreign-npx.js")), realpathSync(npx)),
    );
    const cli = join(root, "node_modules/vize/bin/vize"),
      shim = join(root, "node_modules/.bin/vize");
    write(cli, "authenticated-cli");
    write(
      join(root, "node_modules/vize/package.json"),
      JSON.stringify({ bin: { vize: "bin/vize" } }),
    );
    mkdirSync(dirname(shim), { recursive: true });
    symlinkSync("../vize/bin/vize", shim);
    const before = projectBin(root, cli);
    assert.equal(before.resolved, cli);
    rmSync(shim);
    symlinkSync(join(root, "foreign-npx.js"), shim);
    assert.throws(
      () => projectBin(root, cli),
      assertionData(realpathSync(join(root, "foreign-npx.js")), realpathSync(cli)),
    );
    rmSync(shim);
    write(shim, "foreign shell shim");
    assert.throws(() => projectBin(root, cli), assertionData(false, true));
  }));

test("a semantic pass followed by changed installed bytes leaves every saved acceptance false", () =>
  scratch((root) => {
    const original = input(),
      record = recorder(root);
    const packet = (label: string, value: unknown, text = false) =>
      retain(record, label, ["npx", "vize", "check"], root, "start", {
        pid: 123,
        status: text ? 1 : (value as { errorCount: number }).errorCount,
        signal: null,
        stdout: Buffer.from(
          text
            ? "src/plain.ts\nerror:5:7 [TS2322] Type 'string' is not assignable to type 'number'.\n"
            : JSON.stringify(value),
        ),
        stderr: Buffer.from(
          expectedProgress(root, Number((value as { fileCount?: number } | null)?.fileCount ?? 3)),
        ),
      });
    const outcomes = cases.map((id) => {
      const files = sources(original, "after", id),
        wanted = expected(original, id);
      return {
        id,
        files,
        before: [],
        after: [],
        packet: packet(id, wanted),
        parsed: wanted,
        parseError: null,
        expected: wanted,
        expectedStatus: Number(wanted.errorCount),
      };
    });
    const text = packet("text", null, true);
    const observation = { textBefore: [], textAfter: [], text, outcomes };
    const before = recorder(join(root, "before"));
    mkdirSync(before.root);
    const after = recorder(join(root, "after"));
    mkdirSync(after.root);
    const failedBefore = { ...observation, text: { ...text, status: 2 } };
    assert.throws(() =>
      assessCaptured(
        [
          { role: "before", outcomes: failedBefore, owned: before },
          { role: "after", outcomes: observation, owned: after },
        ],
        record,
      ),
    );
    assert.equal(
      JSON.parse(readFileSync(join(after.root, "semantic-assessment.json"), "utf8")).semanticMatch,
      true,
    );
    assess("after", observation, record);
    const file = join(root, "image.bin");
    write(file, "before");
    const images = [fileImage(file)];
    write(file, "mutate");
    assert.throws(() => assertCustody(images), assertionData(fileImage(file), images[0]));
    const saved = JSON.parse(readFileSync(join(root, "semantic-assessment.json"), "utf8"));
    assert.equal(saved.publicAcceptance, false);
    assert.equal(saved.semanticMatch, true);
    assert.ok(readFileSync(join(root, "Original.stdout.raw")).length > 0);
  }));

test("authenticated loader prerequisites execute zero probes for a local native or module shadow", () =>
  scratch((root) => {
    const version = profiles.after.version,
      modules = join(root, "node_modules");
    const core = join(modules, "@vizejs/native"),
      platform = join(modules, "@vizejs/native-linux-x64-gnu"),
      cli = join(modules, "vize");
    const repo = fileURLToPath(new URL("../../", import.meta.url));
    const materialize = (directory: string, name: string, main: string): PackageBase => {
      write(join(directory, "package.json"), JSON.stringify({ name, version, main }));
      return {
        name,
        version,
        directory,
        resolved: "https://registry.npmjs.org/a.tgz",
        integrity: "integrity",
        archiveSha512: "sha512",
        files: [fileImage(join(directory, "package.json"))],
      };
    };
    const corePackage = materialize(core, "@vizejs/native", "index.js");
    for (const member of nativeLoaderProgram.modules) {
      const body = execFileSync("git", ["show", ":npm/native/" + member.name], { cwd: repo });
      assert.equal(body.length, member.bytes);
      assert.equal(sha256(body), member.sha256);
      write(join(core, member.name), body);
      corePackage.files.push(fileImage(join(core, member.name)));
    }
    const platformPackage = materialize(platform, "@vizejs/native-linux-x64-gnu", "native.node");
    write(join(platform, "native.node"), "inert");
    const binary = fileImage(join(platform, "native.node"));
    platformPackage.files.push(binary);
    const cliPackage = materialize(cli, "vize", "bin/vize");
    write(join(cli, "bin/vize"), "inert-cli");
    cliPackage.files.push(fileImage(join(cli, "bin/vize")));
    const sdk = Array.from({ length: 10 }, (_, index) => {
      const file = join(cli, "sdk-" + index + ".d.ts");
      write(file, "export {};\n");
      return fileImage(file);
    });
    cliPackage.files.push(...sdk);
    const receipt = Buffer.from(
      JSON.stringify({
        schema: "vize.platformVsix.v1",
        source: profiles.after.candidate,
        version,
        provider: {
          source: profiles.after.candidate,
          releaseVersion: version,
          targetPlatform: "linux-x64",
          nodeFile: "native.node",
          nativeBuildToken: "1".repeat(32),
          files: [{ path: "native.node", bytes: binary.bytes, sha256: binary.sha256 }],
        },
      }),
    );
    const capsule = {
      receipts: [
        {
          rawBase64: receipt.toString("base64"),
          sha256: sha256(receipt),
          origin: { runId: profiles.after.run },
        },
      ],
      npmSRI: Object.fromEntries(
        ["@vizejs/native", "@vizejs/native-linux-x64-gnu"].map((name) => [
          name,
          { integrity: "integrity", tarball: "https://registry.npmjs.org/a.tgz" },
        ]),
      ),
    };
    const request = {
      root,
      packages: [cliPackage, corePackage, platformPackage],
      sdk,
      authority: {
        version,
        candidate: profiles.after.candidate,
        source: profiles.after.source,
        workflowHead: profiles.after.source,
        run: profiles.after.run,
        capsule,
        sdk,
        observations: [],
        produced: null,
      },
      env: {},
      sink: () => {},
      host: {
        platform: "linux" as const,
        arch: "x64" as const,
        musl: false,
        classifier: "FILESYSTEM" as const,
        filesystemSha256: "ldd",
      },
    };
    let called = 0;
    const probe = () => {
      called++;
      return "inert callback; no native loading";
    };
    assert.equal(
      probeAfterNativePrerequisite(request, probe).result,
      "inert callback; no native loading",
    );
    assert.equal(called, 1);
    called = 0;
    const local = join(core, "vize-vitrine.linux-x64-gnu.node");
    write(local, "unshipped");
    assert.throws(() => probeAfterNativePrerequisite(request, probe), assertionData(0, 1));
    assert.equal(called, 0);
    rmSync(local);
    const shadow = join(core, "native-binding");
    write(shadow, "unshipped module");
    assert.throws(() => probeAfterNativePrerequisite(request, probe));
    assert.equal(called, 0);
    rmSync(shadow);
    const foreign = join(cli, "node_modules/@vizejs/native/index.js");
    write(foreign, "foreign package");
    assert.throws(() => probeAfterNativePrerequisite(request, probe));
    assert.equal(called, 0);
  }));

test("exact nonquiet progress refuses extras/root/count/order and final custody is retained before verdict", () =>
  scratch((root) => {
    const progress = expectedProgress(root, 3);
    assert.equal(
      progress,
      "Building Corsa virtual project for 3 files under " +
        root +
        "...\nRunning Corsa diagnostics for 3 files...\n",
    );
    const bad = [
      progress + "backend failed\n",
      progress + "warning: degraded\n",
      progress.replace(root, root + "/foreign"),
      progress.replaceAll("3 files", "4 files"),
      progress.split("\n").slice(0, 2).reverse().join("\n") + "\n",
    ];
    const oracle = expected(input(), "Original");
    for (const [index, stderr] of bad.entries()) {
      const packet = retain(
        recorder(root),
        "progress-negative-" + index,
        ["npx", "vize", "check", "-f", "json"],
        root,
        "start",
        {
          pid: 1,
          status: 1,
          signal: null,
          stdout: Buffer.from(JSON.stringify(oracle)),
          stderr: Buffer.from(stderr),
        },
      );
      assert.throws(() => assessJson(packet, oracle, true), assertionData(stderr, progress));
    }
    const beforeRoot = join(root, "before"),
      afterRoot = join(root, "after");
    mkdirSync(beforeRoot);
    mkdirSync(afterRoot);
    const file = join(root, "image"),
      manager = join(root, "manager");
    write(file, "before");
    write(manager, "manager");
    const old = [fileImage(file)],
      managed = [fileImage(manager)];
    write(file, "mutated");
    const failures = retainFinalCustody([
      {
        role: "before",
        owned: recorder(beforeRoot),
        images: old,
        manager: managed,
        verifyRoute: () => {},
      },
      {
        role: "after",
        owned: recorder(afterRoot),
        images: [fileImage(file)],
        manager: managed,
        verifyRoute: () => {},
      },
    ]);
    assert.ok(failures[0]);
    assert.equal(failures[1], null);
    assert.equal(
      JSON.parse(readFileSync(join(beforeRoot, "custody-after.json"), "utf8")).unchanged,
      false,
    );
    const saved = JSON.parse(readFileSync(join(afterRoot, "custody-after.json"), "utf8"));
    assert.equal(saved.unchanged, true);
    assert.equal(saved.publicAcceptance, false);
  }));
