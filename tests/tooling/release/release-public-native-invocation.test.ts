import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  assertPublishedUnicode,
  prepareUnicodeSource,
  unicodeCases,
  unicodeProcessFailure,
} from "../../../tools/support/release/public_acceptance/unicode.ts";
import {
  derivePublicationPlan,
  readRawBlob,
} from "../../../tools/support/release/public_acceptance/plan.ts";
import { readRepoFile } from "../support/github-workflows.ts";
import { sourceFixture } from "../support/release-public-acceptance-fixtures.ts";

const legacy =
  "run: node tools/support/release/publish_npm_package_dirs/main.ts npm/native/npm --provenance";
const concurrent =
  'run: node tools/support/release/publish_npm_package_dirs/main.ts npm/native/npm --concurrency 8 --provenance --receipt "$RUNNER_TEMP/native-platform-publish.json"';

type Fixture = ReturnType<typeof sourceFixture>;

function commitInvocation(source: Fixture, invocation: string) {
  const workflow = readRawBlob(source.root, source.head, ".github/workflows/release.yml");
  assert.equal(workflow.toString().split(legacy).length, 2);
  source.git("reset", "--soft", source.cut);
  source.write(".github/workflows/release.yml", workflow.toString().replace(legacy, invocation));
  source.git("add", ".");
  source.git("commit", "-qm", "authored native invocation H");
  return source.git("rev-parse", "HEAD");
}

function targets(plan: ReturnType<typeof derivePublicationPlan>) {
  return {
    version: plan.version,
    npm: plan.npm,
    crates: plan.crates,
    editor: plan.editor,
    githubAssets: plan.githubAssets,
  };
}

test("legacy native source invocation keeps the original complete plan unchanged", (t) => {
  const source = sourceFixture(t);
  assert.deepEqual(derivePublicationPlan(source.root, source.head), source.plan);
  assert.deepEqual(source.plan.npm, [
    { name: "@vizejs/native", version: "0.438.0" },
    { name: "@vizejs/mcp-musea", version: "0.438.0" },
    { name: "@vizejs/native-darwin-arm64", version: "0.438.0" },
    { name: "@vizejs/native-linux-x64-gnu", version: "0.438.0" },
  ]);
});

test("the exact current eight-child invocation derives the same authored native targets", (t) => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const actual = workflow
    .split("\n")
    .filter(
      (line) =>
        !line.trimStart().startsWith("#") &&
        /tools\/support\/release\/publish_npm_package_dirs\/main\.ts\s/.test(line),
    );
  assert.deepEqual(
    actual.map((line) => line.trim()),
    [concurrent],
  );
  const source = sourceFixture(t);
  const head = commitInvocation(source, actual[0].trim());
  const plan = derivePublicationPlan(source.root, head);
  assert.equal(plan.parentCut, source.cut);
  assert.deepEqual(targets(plan), targets(source.plan));
  const authority = plan.authority.blobs.find(
    (blob) => blob.path === ".github/workflows/release.yml",
  );
  assert.equal(authority?.oid, source.git("rev-parse", `${head}:.github/workflows/release.yml`));
});

test("native source commands refuse every unreviewed flag, path, shell suffix and duplicate", (t) => {
  const source = sourceFixture(t);
  const hostile = [
    concurrent.replace("--concurrency 8", "--concurrency 0"),
    concurrent.replace("--concurrency 8", "--concurrency 3"),
    concurrent.replace("--concurrency 8", "--concurrency 5"),
    concurrent.replace("--concurrency 8", "--concurrency 8oops"),
    concurrent.replace("--provenance ", ""),
    concurrent.replace("npm/native/npm", "npm/other/npm"),
    concurrent.replace("native-platform-publish.json", "other.json"),
    concurrent.replace(
      '"$RUNNER_TEMP/native-platform-publish.json"',
      "$RUNNER_TEMP/native-platform-publish.json",
    ),
    concurrent.replace("--receipt", "--ignore-failure --receipt"),
    concurrent + " --retries 1",
    concurrent + " --unknown",
    concurrent + " && true",
    concurrent + "; true",
    legacy + " --ignore-failure",
    legacy + " --concurrency 8",
    "# " + concurrent,
    legacy + "\n      - " + concurrent,
    concurrent + "\n      - " + concurrent,
    legacy + "\n      - " + concurrent + " --ignore-failure",
  ];
  for (const invocation of hostile) {
    const head = commitInvocation(source, invocation);
    assert.throws(
      () => derivePublicationPlan(source.root, head),
      /(?:one native target publication required|unsupported native target publication command)/,
      invocation,
    );
  }
});

test("sources from before the TypeScript port derive the same plan from MoonBit publishers", (t) => {
  const source = sourceFixture(t);
  const workflow = readRawBlob(source.root, source.head, ".github/workflows/release.yml")
    .toString()
    .replaceAll(
      "node tools/support/release/publish_npm_package/main.ts ",
      "moon run --target native tools/moon/cmd/publish_npm_package -- ",
    )
    .replace(
      "node tools/support/release/publish_npm_package_dirs/main.ts ",
      "moon run --target native tools/moon/cmd/publish_npm_package_dirs -- ",
    );
  source.git("reset", "--soft", source.cut);
  source.git("rm", "-q", "--cached", "tools/support/release/publish_crates/main.ts");
  source.write(".github/workflows/release.yml", workflow);
  source.write(
    "tools/moon/cmd/publish_crates/main.mbt",
    'let published_crates : Array[String] = [\n  "vize_l0",\n  "vize_musea",\n]\n',
  );
  source.git("add", ".github/workflows/release.yml", "tools/moon/cmd/publish_crates/main.mbt");
  source.git("commit", "-qm", "MoonBit-era H");
  const head = source.git("rev-parse", "HEAD");
  const plan = derivePublicationPlan(source.root, head);
  assert.deepEqual(targets(plan), targets(source.plan));
  assert.ok(plan.authority.blobs.some((blob) => blob.path.endsWith("publish_crates/main.mbt")));
});

// Inert filesystem/schema controls: no NAPI, compiler, generated module or runtime executes.
function unicodeReceiptFixture(t: { after(callback: () => void): void }) {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-unicode-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
  const write = (file: string, bytes: string) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes);
  };
  const consumer = path.join(directory, "consumer"),
    output = path.join(directory, "output");
  const read = (file: string) =>
    Buffer.from(file.endsWith("package.json") ? '{"private":true}' : `// frozen H: ${file}\n`);
  const source = prepareUnicodeSource(read, consumer, "a".repeat(40));
  const packageFile = (name: string) => path.join(consumer, "node_modules", name, "package.json");
  const vue = packageFile("vue-ssr-css-vars-oracle"),
    sfc = packageFile("@vue/compiler-sfc"),
    vapor = packageFile("@vue/compiler-vapor");
  const dev = path.join(path.dirname(vue), "dist/vue.runtime-with-vapor.esm-browser.js"),
    prod = dev.replace(".js", ".prod.js");
  const parser = packageFile("@babel/parser"),
    core = packageFile("@babel/core");
  const names = ["vue", "@vue/compiler-sfc", "@vue/compiler-vapor", "@babel/parser", "@babel/core"];
  const packageFiles = [vue, sfc, vapor, parser, core];
  for (const [index, file] of packageFiles.entries())
    write(
      file,
      JSON.stringify({ name: names[index], version: index < 3 ? "3.6.0-rc.10" : "7.29.0" }),
    );
  for (const file of [dev, prod]) write(file, "// inert runtime bytes\n");
  write(
    path.join(consumer, "package.json"),
    JSON.stringify({
      dependencies: {
        "vue-ssr-css-vars-oracle": "npm:vue@3.6.0-rc.10",
        "@babel/core": "7.29.0",
        "@babel/parser": "7.29.0",
      },
    }),
  );
  const packages = packageFiles.map((file) => {
    const actual = JSON.parse(fs.readFileSync(file, "utf8"));
    return {
      location: path.relative(consumer, path.dirname(file)).split(path.sep).join("/"),
      name: actual.name,
      version: actual.version,
      resolved: `https://registry.npmjs.org/${actual.name}/-/inert.tgz`,
      integrity: "sha512-aW5lcnQ=",
    };
  });
  write(
    path.join(consumer, "package-lock.json"),
    JSON.stringify({
      lockfileVersion: 3,
      packages: Object.fromEntries(
        packages.map(({ location, name: _name, ...entry }) => [location, entry]),
      ),
    }),
  );
  const providers = {
    lockSha256: hash(fs.readFileSync(path.join(consumer, "package-lock.json"))),
    packages,
    files: [vue, sfc, vapor, dev, prod, parser, core].map((file) => ({
      file,
      sha256: hash(fs.readFileSync(file)),
    })),
  };
  const frames: Parameters<typeof assertPublishedUnicode>[0]["frames"] = [];
  for (const production of [false, true])
    for (const entry of unicodeCases) {
      const source = `<script setup vapor>\nconst items = [{ id: 1 }]\n</script>\n<template>${entry.template}</template>\n`;
      const code = `export default {}; // inert ${entry.name}`;
      write(
        path.join(output, "native-unicode", `${entry.name}-${production}.compile.json`),
        JSON.stringify({
          source,
          options: {
            filename: "App.vue",
            mode: "module",
            vapor: true,
            scopeId: "probe",
            isProd: production,
            inlineTemplate: production,
          },
          result: { code, errors: [], warnings: [] },
        }),
      );
      for (const official of [false, true]) {
        const stem = `${entry.name}-${production}-${official}`;
        const input = JSON.stringify({
          backend: "vapor",
          code,
          child: "export default {};",
          production,
          unicodeKeyScope: {
            source,
            official,
            samples: [{ args: entry.args, value: entry.value }],
          },
        });
        const stdout = JSON.stringify([
          { tree: entry.texts.map((text) => ({ children: [text] })), namespaces: [], events: [] },
          { tree: [], namespaces: [], events: [] },
        ]);
        const stderr =
          [
            {
              unicodeKeyScope: "module",
              authority: official ? "official-stock" : "fresh-native",
              source,
              code,
              production,
              compilerVersion: "3.6.0-rc.10",
              vaporVersion: "3.6.0-rc.10",
              runtimeVersion: "3.6.0-rc.10",
              runtimePath: production ? prod : dev,
              keyFunctions: [{ code: "(value) => value", params: [], body: {} }],
            },
            { unicodeKeyScope: "key-values", values: [entry.value] },
            { unicodeKeyScope: "dom", diagnostics: [] },
          ]
            .map((row) => JSON.stringify(row))
            .join("\n") + "\n";
        for (const [suffix, bytes] of [
          ["input", input],
          ["stdout", stdout],
          ["stderr", stderr],
        ])
          write(path.join(output, "native-unicode", `${stem}.${suffix}.bin`), bytes);
        const frame = {
          name: entry.name,
          production,
          official,
          stem,
          inputSha256: hash(input),
          stdoutSha256: hash(stdout),
          stderrSha256: hash(stderr),
          pid: frames.length + 1,
          status: 0,
          signal: null,
          processError: null,
        };
        write(path.join(output, "native-unicode", `${stem}.process.json`), JSON.stringify(frame));
        frames.push(frame);
      }
    }
  const receipt = {
    schema: "vize-public-unicode-v1" as const,
    issue: 9493 as const,
    version: "0.438.0",
    head: source.head,
    source,
    providers,
    frames,
    success: true as const,
  };
  const check = (candidate = receipt) =>
    assertPublishedUnicode(candidate, source, "0.438.0", consumer, output);
  return { receipt, check, source, consumer, output, read, write };
}

test("the public native seal requires all twenty original Unicode frames and joined whole outputs", (t) => {
  const fixture = unicodeReceiptFixture(t);
  fixture.check();
  for (const mutate of [
    (value: typeof fixture.receipt) => value.frames.pop(),
    (value: typeof fixture.receipt) => value.frames.reverse(),
    (value: typeof fixture.receipt) => {
      value.frames[0].status = 1;
    },
    (value: typeof fixture.receipt) => {
      value.frames[0].signal = "SIGTERM";
    },
    (value: typeof fixture.receipt) => {
      value.frames[0].stdoutSha256 = "f".repeat(64);
    },
    (value: typeof fixture.receipt) => {
      value.head = "b".repeat(40);
    },
  ]) {
    const candidate = structuredClone(fixture.receipt);
    mutate(candidate);
    assert.throws(() => fixture.check(candidate));
  }
  const frame = fixture.receipt.frames[0];
  fixture.write(path.join(fixture.output, "native-unicode", `${frame.stem}.stderr.bin`), "");
  assert.throws(() => fixture.check());
});

test("frozen H support and same-install official runtime bytes remain mandatory at the public seal", (t) => {
  const fixture = unicodeReceiptFixture(t);
  assert.throws(
    () => prepareUnicodeSource(fixture.read, fixture.consumer, fixture.source.head),
    /EEXIST/,
  );
  const helper = path.join(fixture.consumer, "tests/tooling/support/vapor-unicode-key-scope.ts");
  fixture.write(helper, "// changed during installation\n");
  assert.throws(() => fixture.check());
  fixture.write(
    helper,
    fixture.read("tests/tooling/support/vapor-unicode-key-scope.ts").toString(),
  );
  fixture.write(fixture.receipt.providers.files[3].file, "// changed official runtime\n");
  assert.throws(() => fixture.check());
});

test("public Unicode process refusals retain the original Error fields, cause and non-Error values", () => {
  const cause = new Error("underlying spawn refusal");
  const failure = Object.assign(new Error("original process refusal", { cause }), {
    code: "ENOENT",
    errno: -2,
    syscall: "spawn",
    path: "/inert/node",
    spawnargs: ["/inert/observer.mjs"],
  });
  const packet = unicodeProcessFailure(failure)!;
  assert.equal(packet.name, failure.name);
  assert.equal(packet.message, failure.message);
  assert.equal(packet.stack, failure.stack);
  for (const value of [
    "ENOENT",
    "-2",
    "spawn",
    "/inert/node",
    "/inert/observer.mjs",
    cause.message,
  ]) {
    assert.ok(packet.raw.includes(value));
    assert.ok(packet.ownProperties!.includes(value));
  }
  assert.deepEqual(unicodeProcessFailure(42), {
    type: "number",
    name: null,
    message: null,
    stack: null,
    ownProperties: null,
    raw: "42",
  });
  assert.equal(unicodeProcessFailure(undefined), null);
});
