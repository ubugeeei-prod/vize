/** Whole public-contract packets from two source images and actual providers. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { observe, type ProcessPacket } from "./native-capture.ts";

const args = new Map<string, string>();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv.at(i),
    value = process.argv.at(i + 1);
  assert.ok(key && value && !args.has(key));
  args.set(key, value);
}
const required = (key: string) => {
  const value = args.get(key);
  assert.ok(value, key);
  return value;
};
const consumer = fs.realpathSync(required("--consumer"));
const output = path.resolve(required("--output"));
const source = required("--source");
const baselineSource = required("--baseline-source");
assert.match(source, /^[a-f0-9]{40}$/u);
assert.match(baselineSource, /^[a-f0-9]{40}$/u);
assert.equal(fs.existsSync(output), false);
fs.mkdirSync(output, { recursive: true });
const binaries = {
  baseline: fs.realpathSync(required("--baseline")),
  candidate: fs.realpathSync(required("--candidate")),
};
const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(
  repository,
  "tests/_fixtures/differential/typechecker/empty-generic-public-contract-10540",
);
const files = ["App.vue", "Base.vue", "Child.vue", "Fragment.vue", "Generic.vue"];
const read = (name: string) => fs.readFileSync(path.join(fixture, `${name}.txt`), "utf8");
const config = JSON.parse(read("tsconfig.json"));
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const digest = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const packageRoot = (name: string) => fs.realpathSync(path.join(consumer, "node_modules", name));
const versions = {
  vize: "0.569.0",
  typescript: "6.0.3",
  "typescript-native": "7.1.0-beta",
  vue: "3.6.0-beta.10",
  "vue-tsc": "3.3.11",
  "vue-component-type-helpers": "3.3.11",
};
for (const [name, version] of Object.entries(versions))
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(packageRoot(name), "package.json"), "utf8")).version,
    version,
  );
const native = fs.realpathSync(
  path.join(
    path.dirname(
      createRequire(path.join(packageRoot("vize"), "package.json")).resolve(
        `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
      ),
    ),
    "lib/tsc",
  ),
);
const official = fs.realpathSync(
  (
    await import(
      pathToFileURL(path.join(packageRoot("typescript-native"), "lib/getExePath.js")).href
    )
  ).default(),
);
const reference = path.join(packageRoot("vue-tsc"), "bin/vue-tsc.js");
const env = Object.fromEntries(
  Object.entries(process.env).filter(
    ([key]) =>
      !/^(VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/u.test(key) &&
      !["NODE_OPTIONS", "NODE_PATH"].includes(key),
  ),
);
env.PATH = path.dirname(process.execPath) + path.delimiter + (env.PATH ?? "");
save(path.join(output, "providers.json"), {
  source,
  baselineSource,
  node: process.version,
  scope: "Exact source qualification; not a publication or raw RPC witness",
  packages: Object.keys(versions).map((name) => ({
    name,
    package: JSON.parse(fs.readFileSync(path.join(packageRoot(name), "package.json"), "utf8")),
  })),
  physical: [...Object.values(binaries), native, official, reference, process.execPath].map(
    (file) => ({ file, sha256: digest(file) }),
  ),
});
for (const [name, binary, version] of [
  ["native", native, "Version 7.0.2\n"],
  ["official", official, "Version 7.1.0-beta\n"],
]) {
  const result = observe([binary!, "--version"], consumer, path.join(output, name!), env);
  assert.deepEqual(
    [result.status, result.signal, result.error, result.stdout, result.stderr],
    [0, null, null, version, ""],
  );
}
const cases = [
  { id: "absent", attribute: "" },
  { id: "bare", attribute: " generic" },
  { id: "quoted-empty", attribute: ' generic=""' },
  { id: "single-empty", attribute: " generic=''" },
  { id: "whitespace", attribute: ' generic=" \t "' },
  { id: "unicode-whitespace", attribute: ' generic="\u00a0"' },
  { id: "valid", attribute: ' generic="T"' },
  { id: "body-error", attribute: " generic" },
];
type Side = { side: "baseline" | "candidate"; check: ProcessPacket; mapper: ProcessPacket };
const observations: { id: string; kind: string; sides: Side[]; reference: ProcessPacket }[] = [];
for (const test of cases)
  for (const newline of ["\n", "\r\n"])
    for (const unicode of [false, true]) {
      const id = `${test.id}-${newline === "\n" ? "lf" : "crlf"}-${unicode ? "unicode" : "plain"}`;
      const root = path.join(output, "projects", id);
      fs.mkdirSync(root, { recursive: true });
      const inputs = Object.fromEntries(
        files.map((name) => {
          let text = read(name);
          if (["Fragment.vue", "Generic.vue"].includes(name)) {
            text = text.replace(' lang="ts" generic>', ` lang="ts"${test.attribute}>`);
            if (unicode) text = "<!-- 日本 🧭 -->" + text;
          }
          if (name === "App.vue" && test.id === "body-error")
            text = text.replace('lang="ts">', "lang=\"ts\">\nconst own: number = 'wrong';");
          return [name, text.replaceAll("\n", newline)];
        }),
      );
      const sides: Side[] = [];
      for (const side of ["baseline", "candidate"] as const) {
        const project = path.join(root, side);
        fs.mkdirSync(project);
        for (const [name, text] of Object.entries(inputs))
          fs.writeFileSync(path.join(project, name), text);
        save(path.join(project, "tsconfig.json"), config);
        save(path.join(project, "tsconfig.mapper.json"), {
          ...config,
          contentMappers: [{ package: "vize", extensions: [".vue"] }],
        });
        const deps = path.join(project, "node_modules");
        fs.mkdirSync(path.join(deps, "vize"), { recursive: true });
        for (const name of ["vue", "vue-component-type-helpers"])
          fs.symlinkSync(packageRoot(name), path.join(deps, name), "dir");
        save(path.join(deps, "vize/package.json"), {
          name: "vize",
          private: true,
          version: `0.0.0-source-${side === "baseline" ? baselineSource : source}`,
          typescript: {
            contentMapper: {
              exec: [binaries[side], "content-mapper"],
              dynamicConfig: true,
              compilerOptions: ["checkJs", "noUnusedLocals"],
            },
          },
        });
        save(path.join(project, "startup-configuration.json"), {
          manifest: JSON.parse(fs.readFileSync(path.join(deps, "vize/package.json"), "utf8")),
          inputs: Object.entries(inputs).map(([file, text]) => ({
            file,
            sha256: createHash("sha256").update(text).digest("hex"),
          })),
          custody: "Configuration evidence; not raw RPC evidence",
        });
        const check = observe(
          [
            binaries[side],
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--quiet",
            "--corsa-path",
            native,
          ],
          project,
          path.join(project, "check"),
          env,
        );
        const mapper = observe(
          [
            official,
            "--runExternalCode",
            "--noEmit",
            "-p",
            "tsconfig.mapper.json",
            "--pretty",
            "false",
          ],
          project,
          path.join(project, "mapper"),
          env,
        );
        fs.rmSync(deps, { recursive: true });
        sides.push({ side, check, mapper });
      }
      const project = path.join(root, "reference");
      fs.mkdirSync(project);
      for (const [name, text] of Object.entries(inputs))
        fs.writeFileSync(path.join(project, name), text);
      save(path.join(project, "tsconfig.json"), config);
      fs.symlinkSync(
        path.join(consumer, "node_modules"),
        path.join(project, "node_modules"),
        "dir",
      );
      const result = observe(
        [process.execPath, reference, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
        project,
        path.join(project, "reference"),
        env,
      );
      fs.rmSync(path.join(project, "node_modules"));
      observations.push({ id, kind: test.id, sides, reference: result });
    }
save(path.join(output, "inventory.json"), {
  projects: observations.length,
  topLevelProcesses: observations.length * 5,
  ids: observations.map((row) => row.id),
  completeCaptureBeforeAssertions: true,
});
assert.equal(observations.length, 32);
function stable(packet: ProcessPacket) {
  assert.equal(packet.signal, null);
  assert.equal(packet.error, null);
  assert.equal(packet.stderr, "");
}
const wrong = "Type 'string' is not assignable to type 'number'.";
const falseConstraint = "Type 'false' does not satisfy the constraint 'true'.";
for (const row of observations) {
  for (const side of row.sides) {
    stable(side.check);
    stable(side.mapper);
    const body = row.kind === "body-error";
    const nonfunctional =
      row.kind === "absent" || (side.side === "baseline" && row.kind !== "valid");
    const expected = [
      ...(body ? [{ line: 2, column: 7, code: 2322, message: wrong }] : []),
      ...(nonfunctional
        ? [{ line: body ? 12 : 11, column: 31, code: 2344, message: falseConstraint }]
        : []),
    ];
    assert.equal(side.check.status, expected.length ? 1 : 0, row.id);
    assert.deepEqual(
      JSON.parse(side.check.stdout),
      {
        files: files.map((file) => ({
          file,
          diagnostics:
            file === "App.vue"
              ? expected.map((d) => `error:${d.line}:${d.column} [TS${d.code}] ${d.message}`)
              : [],
        })),
        programs: [
          { root: ".", tsconfig: "tsconfig.json", compilerOptions: config.compilerOptions, files },
        ],
        errorCount: expected.length,
        warningCount: 0,
        fileCount: 5,
      },
      `${row.id}/${side.side}: complete source packet`,
    );
    assert.equal(side.mapper.status, expected.length ? 2 : 0, row.id);
    // The mapper reports generated module declarations before setup-body code;
    // the native CLI independently restores authored source order above.
    const mapperExpected = body && nonfunctional ? [expected[1]!, expected[0]!] : expected;
    assert.equal(
      side.mapper.stdout,
      mapperExpected
        .map((d) => `App.vue(${d.line},${d.column}): error TS${d.code}: ${d.message}\n`)
        .join(""),
      `${row.id}/${side.side}: complete mapper packet`,
    );
  }
  stable(row.reference);
  let expected = "";
  if (row.kind === "absent") expected = `App.vue(11,31): error TS2344: ${falseConstraint}\n`;
  else if (row.kind === "body-error") expected = `App.vue(2,7): error TS2322: ${wrong}\n`;
  else if (["quoted-empty", "single-empty", "whitespace", "unicode-whitespace"].includes(row.kind))
    expected =
      "App.vue(10,64): error TS2339: Property '$el' does not exist on type '{}'.\nApp.vue(14,61): error TS2339: Property '$el' does not exist on type '{}'.\nApp.vue(15,16): error TS2339: Property 'run' does not exist on type '{}'.\n";
  assert.equal(row.reference.status, expected ? 2 : 0, row.id);
  assert.equal(
    row.reference.stdout,
    expected,
    `${row.id}: complete independent reference packet; existing empty-value differences remain explicit`,
  );
}
save(path.join(output, "result.json"), {
  source,
  baselineSource,
  projects: 32,
  processes: 160,
  result: "COMPLETE_EXPECTED_VECTORS_PASSED",
  referenceDifference:
    "Quoted-empty and whitespace attributes retain existing three2339 reference differences; parity is not claimed",
  malformedControls:
    "Existing unchanged144 strict syntax/body-error observations are qualified separately on the composed parent/route child",
});
