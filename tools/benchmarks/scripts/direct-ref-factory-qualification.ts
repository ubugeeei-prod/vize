/** One source qualification: focused exhaustive geometry and full cold code parity. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { identify, run, save, hash } from "./canon-instance-directives-io.ts";
import { packet, isExecutableFactory, type CliResult } from "./canon-instance-directives-model.ts";
import { factory, compareCaptures, codeEdits, type Capture } from "./direct-ref-factory-parity.ts";
import { sameSdk, git } from "./direct-ref-factory-source.ts";
import { prepareColdScale } from "./cold-typechecker-scale.ts";

export const inputFiles = [
  ["Resolver.vue.txt", "src/Resolver.vue"],
  ["Parent.vue.txt", "src/Parent.vue"],
  ["names.ts.txt", "src/names.ts"],
  ["tsconfig.json.txt", "tsconfig.json"],
] as const;
export function compareCli(before: CliResult, after: CliResult) {
  const expected = structuredClone(before);
  let activeFiles = 0,
    uses = 0;
  assert.equal(after.files.length, before.files.length);
  expected.files.forEach((file, index) => {
    const current = after.files[index];
    assert.equal(current.file, file.file);
    if (typeof file.virtualTs === "string") {
      assert(typeof current.virtualTs === "string");
      const bytes = Buffer.from(file.virtualTs),
        at = bytes.indexOf("  " + factory);
      let delta = 0;
      if (at >= 0) {
        // This closed population has exactly one prior static Vue import.
        const imports = [
          ...bytes
            .subarray(0, at)
            .toString()
            .matchAll(/\bfrom\s+["']([^"']+\.vue\.ts)["']/gu),
        ];
        assert.equal(imports.length, 1);
        delta = 3;
      }
      const law = codeEdits(file.virtualTs, current.virtualTs, delta);
      if (law.uses) activeFiles++;
      uses += law.uses;
      file.virtualTs = current.virtualTs;
    }
  });
  assert(
    isDeepStrictEqual(after, expected),
    "complete CLI fields/diagnostics/order/unknown metadata",
  );
  return { activeFiles, uses };
}
export function exactDebug(result: CliResult, shared: string, root: string) {
  const files = result.files
    .filter((f) => typeof f.virtualTs === "string")
    .map((f) => ({ ...f, absolute: path.resolve(root, f.file) }));
  assert.equal(new Set(files.map((f) => f.absolute)).size, files.length);
  files.sort((a, b) => Buffer.compare(Buffer.from(a.absolute), Buffer.from(b.absolute)));
  return Buffer.from(
    `\n=== __vize_helpers.d.ts ===\n${shared}\n` +
      files.map((f) => `\n=== ${f.absolute} ===\n${f.virtualTs}\n`).join(""),
  );
}
export function expectedErrors(parent: string, kind: "number" | "any" | null) {
  if (kind === null) return [];
  const names = kind === "number" ? ["numeric"] : ["numericNoAny", "textualNoAny"];
  return names.map((name) => {
    const rows = parent
      .split("\n")
      .flatMap((text, index) =>
        text.trimStart().startsWith(`const ${name}:`) ? [{ text, index }] : [],
      );
    assert.equal(rows.length, 1);
    const { text, index } = rows[0];
    return {
      file: "src/Parent.vue",
      line: index + 1,
      column: text.indexOf(name) + 1,
      code: 2322,
      message:
        kind === "number"
          ? "Type 'number' is not assignable to type 'string'."
          : "Type 'true' is not assignable to type 'false'.",
    };
  });
}
export async function qualify(base: string, head: string, provider: string, out: string) {
  fs.mkdirSync(out, { recursive: true });
  const sources = sameSdk(base, head),
    fixture = path.join(head, "tests/_fixtures/differential/typechecker/direct-ref-factory");
  const inputs = inputFiles.map(([pin, file]) => ({
    file,
    wholeUtf8: fs.readFileSync(path.join(fixture, pin), "utf8"),
  }));
  const captures = (["base", "head"] as const).map(
    (role) =>
      JSON.parse(fs.readFileSync(path.join(out, role + "-projection.json"), "utf8")) as Capture,
  );
  for (const capture of captures) assert.deepEqual(capture.inputs, inputs);
  const geometry = compareCaptures(captures[0], captures[1]);
  const observer = "crates/vize_canon/tests/direct_ref_factory_projection.rs";
  assert.equal(
    identify(path.join(base, observer)).sha256,
    identify(path.join(head, observer)).sha256,
  );
  assert.equal(git(base, "status", "--porcelain", "--untracked-files=all"), "?? " + observer);
  assert.equal(git(base, "diff", "--name-only", "--", "crates/vize_canon/src"), "");
  const build = JSON.parse(fs.readFileSync(path.join(out, "base-build.json"), "utf8")) as {
    source: string;
    tree: string;
    image: { path: string; sha256: string };
    cleanBeforeObserverCopy: boolean;
    observerImportedYet: boolean;
  };
  assert.equal(build.source, git(base, "rev-parse", "HEAD"));
  assert.equal(build.tree, git(base, "rev-parse", "HEAD^{tree}"));
  assert(build.cleanBeforeObserverCopy && !build.observerImportedYet);
  assert.equal(identify(build.image.path).sha256, build.image.sha256);
  const images = [build.image.path, path.join(process.env.RUNNER_TEMP!, "slot-head")];
  const observerImages = (["base", "head"] as const).map((role) => {
    const deps = path.join(
      process.env.RUNNER_TEMP!,
      role === "base" ? "direct-factory-base-target" : "slot-head-target",
      "ci-opt/deps",
    );
    const matches = fs
      .readdirSync(deps)
      .filter((name) => /^direct_ref_factory_projection-[a-f0-9]+$/u.test(name));
    assert.equal(matches.length, 1);
    return {
      role,
      image: identify(path.join(deps, matches[0])),
      observer: identify(path.join(head, observer)),
    };
  });
  const headBuild = JSON.parse(fs.readFileSync(path.join(out, "../head-build.json"), "utf8")) as {
    source: string;
    sha256: string;
  };
  assert.equal(headBuild.source, git(head, "rev-parse", "HEAD"));
  assert.equal(identify(images[1]).sha256, headBuild.sha256);
  const module: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  assert(isExecutableFactory(module));
  const selected = module.default();
  assert(typeof selected === "string");
  const native = fs.realpathSync(selected);
  assert.equal(
    identify(native).sha256,
    "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
  const identities = [
    ...images,
    native,
    path.join(provider, "package-lock.json"),
    path.join(provider, "node_modules/vue/package.json"),
    path.join(provider, "node_modules/typescript/package.json"),
    path.join(provider, "node_modules/typescript/lib/typescript.js"),
  ].map(identify);
  assert.equal(
    identities[3].sha256,
    "9ca03c3cb103e1ff8d0a8c633145a743a056f21c5a0be56fa633a19541ab5256",
  );
  save(path.join(out, "source-inputs.json"), {
    sources,
    geometry,
    identities,
    observerImages,
    inputs,
    baseBeforeObserverReceipt: identify(path.join(out, "base-build.json")),
    headImageReceipt: identify(path.join(out, "../head-build.json")),
    scope:
      "current source SDK10 equality; installed public 0.603 is an available provider, not current public acceptance",
  });
  const project = path.join(out, "project");
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  for (const input of inputs) fs.writeFileSync(path.join(project, input.file), input.wholeUtf8);
  const originalParent = inputs[1].wholeUtf8,
    originalChild = inputs[0].wholeUtf8;
  const replace = (s: string, a: string, b: string) => {
    assert.equal(s.split(a).length, 2);
    return s.replace(a, b);
  };
  const states = [
    ["valid", originalParent, originalChild, null],
    [
      "wrong-number",
      replace(originalParent, "const numeric: number", "const numeric: string"),
      originalChild,
      "number",
    ],
    ["number-repair", originalParent, originalChild, null],
    [
      "any-poison",
      originalParent,
      replace(originalChild, "current: () => props.value", "current: () => (props.value as any)"),
      "any",
    ],
    ["any-repair", originalParent, originalChild, null],
  ] as const;
  const observations: unknown[] = [];
  function capturePair(
    name: string,
    root: string,
    shared: [string, string],
    wanted: ReturnType<typeof expectedErrors> | null,
    expectedFiles: number | null,
  ) {
    const state = path.join(out, name);
    fs.mkdirSync(state);
    let previous: ReturnType<typeof packet> | undefined;
    for (let index = 0; index < 2; index++) {
      const role = index === 0 ? "base" : "head",
        runtime = path.join(out, "runtime-" + name + "-" + role);
      fs.mkdirSync(path.join(runtime, "xdg"), { recursive: true });
      fs.mkdirSync(path.join(runtime, "tmp"));
      const actual = run(
        state,
        role,
        images[index],
        [
          "check",
          "--no-config",
          "--format",
          "json",
          "--show-virtual-ts",
          "--quiet",
          "--tsconfig",
          "tsconfig.json",
          "--corsa-path",
          native,
        ],
        root,
        {
          ...process.env,
          XDG_CACHE_HOME: path.join(runtime, "xdg"),
          TMPDIR: path.join(runtime, "tmp"),
        },
      );
      const parsed = packet(actual.stdout, actual.status, root);
      assert(
        actual.stdout.equals(Buffer.from(JSON.stringify(parsed.result, null, 2) + "\n")),
        "whole JSON format",
      );
      assert(
        actual.stderr.equals(exactDebug(parsed.result, shared[index], root)),
        "whole raw debug stderr must equal all bodies in source absolute-byte order",
      );
      if (wanted !== null)
        assert.deepEqual(
          parsed.diagnostics.map(({ file, line, column, code, message }) => ({
            file,
            line,
            column,
            code,
            message,
          })),
          wanted,
        );
      if (expectedFiles !== null) {
        assert.equal(parsed.result.fileCount, expectedFiles);
        assert.equal(parsed.result.files.length, expectedFiles);
      }
      let law: ReturnType<typeof compareCli> | null = null;
      if (previous) {
        law = compareCli(previous.result, parsed.result);
        if (expectedFiles === 4323) assert.deepEqual(law, { activeFiles: 622, uses: 622 });
        else assert.deepEqual(law, { activeFiles: 1, uses: 2 });
      }
      observations.push({
        name,
        role,
        status: actual.status,
        diagnostics: parsed.diagnostics,
        stdout: { bytes: actual.stdout.length, sha256: hash(actual.stdout) },
        stderr: { bytes: actual.stderr.length, sha256: hash(actual.stderr) },
        law,
      });
      previous = parsed;
    }
  }
  const shared: [string, string] = [captures[0].sharedPreamble, captures[1].sharedPreamble];
  for (const [name, parent, child, kind] of states) {
    fs.writeFileSync(path.join(project, "src/Parent.vue"), parent);
    fs.writeFileSync(path.join(project, "src/Resolver.vue"), child);
    save(path.join(out, name + "-inputs.json"), {
      parent,
      child,
      config: inputs[3],
      names: inputs[2],
    });
    capturePair(name, project, shared, expectedErrors(parent, kind), null);
    assert.equal(fs.readFileSync(path.join(project, "src/Parent.vue"), "utf8"), parent);
    assert.equal(fs.readFileSync(path.join(project, "src/Resolver.vue"), "utf8"), child);
    for (const input of inputs.slice(2))
      assert.equal(fs.readFileSync(path.join(project, input.file), "utf8"), input.wholeUtf8);
  }
  const corpus = prepareColdScale(path.join(out, "cold-project"), provider, 4323, false);
  assert.equal(corpus.sfcFiles, 2500);
  save(path.join(out, "cold-inputs.json"), corpus);
  capturePair("cold-4323", corpus.app, shared, null, 4323);
  for (const entry of corpus.entries)
    assert.equal(identify(path.join(out, "cold-project", entry.path)).sha256, entry.sha256);
  for (const identity of identities) assert.deepEqual(identify(identity.path), identity);
  save(path.join(out, "qualification.json"), {
    sources,
    geometry,
    observations,
    actualCliProcesses: 12,
    fullPopulation: {
      authoredFiles: 4323,
      sfcs: 2500,
      activeFactoryFiles: 622,
      scope:
        "whole generated code/comments and CLI vectors; exhaustive mapping geometry only the two focused SFCs in two modes",
    },
    nativeLoadedFiles: "unobserved",
    coldRawNativeDiagnostics:
      "unobserved here; historical source597 capture had 624 TS5097 despite authored CLI zero",
    timingComparison: false,
    images: identities.slice(0, 2),
  });
}
if (process.argv[2] === "qualify") {
  assert.equal(process.argv.length, 7);
  await qualify(
    ...(process.argv.slice(3).map((v) => path.resolve(v)) as [string, string, string, string]),
  );
}
