/** Five bounded whole CLI before/head controls over the same pinned project. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { identify, run, save, hash } from "./canon-instance-directives-io.ts";
import { packet, isExecutableFactory, type CliResult } from "./canon-instance-directives-model.ts";
import { readRefFactoryAuthority } from "./ref-factory-authority.ts";
import { factoryCodeEdits } from "./ref-factory-parity.ts";

export async function qualifyRefFactory(
  provider: string,
  baseline: string,
  head: string,
  out: string,
) {
  const authority = readRefFactoryAuthority();
  const fixture = path.resolve("tests/_fixtures/differential/typechecker/shared-ref-factory");
  const originalProbe = fs.readFileSync(path.join(fixture, "Probe.vue.txt"), "utf8"),
    originalChild = fs.readFileSync(path.join(fixture, "Child.vue.txt"), "utf8"),
    config = fs.readFileSync(path.join(fixture, "tsconfig.json.txt"));
  fs.mkdirSync(out, { recursive: true });
  const project = path.join(out, "project");
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  fs.writeFileSync(path.join(project, "tsconfig.json"), config);
  const getExe: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  assert(isExecutableFactory(getExe));
  const selected = getExe.default();
  assert(typeof selected === "string");
  const backend = fs.realpathSync(selected),
    identities = [
      identify(baseline),
      identify(head),
      identify(backend),
      identify(path.join(provider, "package-lock.json")),
      identify(path.join(provider, "node_modules/vue/package.json")),
      identify(path.join(fixture, "Probe.vue.txt")),
      identify(path.join(fixture, "Child.vue.txt")),
      identify(path.join(fixture, "tsconfig.json.txt")),
    ];
  const manifest = JSON.parse(
    fs.readFileSync("tests/_fixtures/differential/typechecker/manifest.json", "utf8"),
  ) as { cases: { id: string; inputs?: { file: string; path: string; sha256: string }[] }[] };
  const pins = manifest.cases.filter(
    (row) => row.id === "typechecker/shared-ref-factory/instantiated-generic-dom",
  );
  assert.equal(pins.length, 1);
  assert.deepEqual(
    pins[0].inputs,
    [
      ["Child.vue.txt", "src/Child.vue"],
      ["Probe.vue.txt", "src/Probe.vue"],
      ["tsconfig.json.txt", "tsconfig.json"],
    ].map(([source, file]) => ({
      file,
      path: "shared-ref-factory/" + source,
      sha256: identify(path.join(fixture, source)).sha256,
    })),
  );
  const wrongProbe = originalProbe.replace("const numeric: number", "const numeric: string");
  assert.notEqual(wrongProbe, originalProbe);
  const anyChild = originalChild.replace(
    "current: () => props.value",
    "current: () => (props.value as any)",
  );
  assert.notEqual(anyChild, originalChild);
  const rows: unknown[] = [];
  for (const [name, probe, child, errorKind] of [
    ["valid", originalProbe, originalChild, null],
    ["wrong-owned-number", wrongProbe, originalChild, "numeric"],
    ["number-repair", originalProbe, originalChild, null],
    ["explicit-any-poison", originalProbe, anyChild, "noAny"],
    ["any-repair", originalProbe, originalChild, null],
  ] as const) {
    const state = path.join(out, name);
    fs.mkdirSync(state);
    fs.writeFileSync(path.join(project, "src/Probe.vue"), probe);
    fs.writeFileSync(path.join(project, "src/Child.vue"), child);
    save(path.join(state, "inputs.json"), {
      files: [
        { file: "src/Probe.vue", wholeUtf8: probe, sha256: hash(probe) },
        { file: "src/Child.vue", wholeUtf8: child, sha256: hash(child) },
        { file: "tsconfig.json", wholeUtf8: config.toString("utf8"), sha256: hash(config) },
      ],
    });
    let previous: ReturnType<typeof packet> | undefined;
    for (const [role, image] of [
      ["base", baseline],
      ["head", head],
    ] as const) {
      const runtime = path.join(out, "runtime");
      fs.rmSync(runtime, { recursive: true, force: true });
      fs.mkdirSync(path.join(runtime, "xdg"), { recursive: true });
      fs.mkdirSync(path.join(runtime, "tmp"));
      const env = {
        ...process.env,
        XDG_CACHE_HOME: path.join(runtime, "xdg"),
        TMPDIR: path.join(runtime, "tmp"),
      };
      const observed = run(
        state,
        role,
        image,
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
          backend,
        ],
        project,
        env,
      );
      const parsed = packet(observed.stdout, observed.status, project);
      for (const [file, body] of [
        ["src/Probe.vue", Buffer.from(probe)],
        ["src/Child.vue", Buffer.from(child)],
        ["tsconfig.json", config],
      ] as const)
        assert.deepEqual(
          fs.readFileSync(path.join(project, file)),
          body,
          "both actual CLI images must consume the identical untouched authored/config bytes",
        );
      assert.deepEqual(
        observed.stdout,
        Buffer.from(JSON.stringify(parsed.result, null, 2) + "\n"),
        "complete JSON formatting must remain exact",
      );
      const helper =
        role === "base" ? authority.preamble.beforeWholeUtf8 : authority.preamble.afterWholeUtf8;
      const dumped =
        `\n=== __vize_helpers.d.ts ===\n${helper}\n` +
        parsed.result.files
          .filter((file) => typeof file.virtualTs === "string")
          .map((file) => `\n=== ${path.resolve(project, file.file)} ===\n${file.virtualTs}\n`)
          .join("");
      assert.deepEqual(
        observed.stderr,
        Buffer.from(dumped),
        "complete raw stderr must be the exact current helper and every JSON virtual-file body",
      );
      if (!previous) previous = parsed;
      else {
        const expected = structuredClone(previous.result) as CliResult;
        assert.equal(parsed.result.files.length, expected.files.length);
        expected.files.forEach((file, index) => {
          const current = parsed.result.files[index];
          assert.equal(file.file, current.file);
          if (typeof file.virtualTs === "string") {
            assert(typeof current.virtualTs === "string");
            const oldFactory = "  " + authority.declaration.wholeUtf8;
            const at = Buffer.from(file.virtualTs).indexOf(oldFactory);
            let importDelta = 0;
            if (at >= 0) {
              assert.equal(path.basename(file.file), "Probe.vue");
              const imports = [
                ...Buffer.from(file.virtualTs)
                  .subarray(0, at)
                  .toString("utf8")
                  .matchAll(/\bfrom\s+["']([^"']+\.vue\.ts)["']/gu),
              ].map((match) => match[1]);
              assert.deepEqual(imports, ["./Child.vue.ts"]);
              importDelta = 3;
            }
            factoryCodeEdits(file.virtualTs, current.virtualTs, oldFactory, importDelta);
            file.virtualTs = current.virtualTs;
          }
        });
        assert.deepEqual(
          parsed.result,
          expected,
          "every code field, complete diagnostics, order and unknown result field must obey the finite law",
        );
        assert.deepEqual(parsed.diagnostics, previous.diagnostics);
      }
      const actual = parsed.diagnostics.map(({ file, line, column, code, message }) => ({
        file,
        line,
        column,
        code,
        message,
      }));
      const line = errorKind
        ? probe.split("\n").findIndex((text) => text.startsWith(`const ${errorKind}:`)) + 1
        : 0;
      const wanted = errorKind
        ? [
            {
              file: "src/Probe.vue",
              line,
              column: 7,
              code: 2322,
              message: `${errorKind === "numeric" ? "Type 'number' is not assignable to type 'string'." : "Type 'true' is not assignable to type 'false'."} (source: ${probe.split("\n")[line - 1].trim()})`,
            },
          ]
        : [];
      assert.deepEqual(actual, wanted, `${name}/${role}: full owned native diagnostic vector`);
      rows.push({
        name,
        role,
        diagnostics: actual,
        stdout: hash(observed.stdout),
        stderr: hash(observed.stderr),
        status: observed.status,
      });
    }
  }
  for (const identity of identities) assert.deepEqual(identify(identity.path), identity);
  save(path.join(out, "native-summary.json"), {
    inputs: {
      probe: identify(path.join(fixture, "Probe.vue.txt")),
      child: identify(path.join(fixture, "Child.vue.txt")),
      config: identify(path.join(fixture, "tsconfig.json.txt")),
    },
    identities,
    rows,
    actualCliProcesses: rows.length,
    actualSourceInputsShared: true,
    rawKept: true,
    finiteCodeAndStderrLawsPassed: true,
    timingComparison: false,
    nativeLoadedFileVector: "unobserved",
    scope:
      "five pinned-project valid/owned-negative/NoAny-poison/repair states on exact source CLI images; not all ref/library shapes or public SDK acceptance",
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  assert.equal(process.argv.length, 6);
  await qualifyRefFactory(
    ...(process.argv
      .slice(2)
      .map(
        (value) =>
          fs.realpathSync(path.dirname(path.resolve(value))) + path.sep + path.basename(value),
      ) as [string, string, string, string]),
  );
}
