import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  asStringArray,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";

const [version, consumer, identityFile] = process.argv.slice(2);
assert.ok(version && /^[0-9]+\.[0-9]+\.[0-9]+$/.test(version));
assert.ok(consumer && isAbsolute(consumer));
assert.ok(identityFile && isAbsolute(identityFile));
assert.equal(process.platform, "linux", "the capsule pins the actual Linux provider");
assert.equal(process.arch, "x64");
assert.equal(existsSync(consumer), false, "acceptance requires a fresh consumer");
const sourceRoot = dirname(fileURLToPath(import.meta.url));
const identity = readJsonRecord(identityFile);
assert.equal(identity.version, version);
assert.equal(identity.requiredSource, "7c2f6f46172404a8f95335a79ee2b95094f6b7ee");
assert.equal(identity.containsRequiredSource, true);
assert.match(asString(identity.sourceSha), /^[0-9a-f]{40}$/);
const sha256 = (bytes: string | Buffer): string => createHash("sha256").update(bytes).digest("hex");
const manifest = readJsonRecord(new URL("./immutable-input-manifest.json.txt", import.meta.url));
for (const [name, entry] of Object.entries(manifest)) {
  assert.equal(name.includes(".."), false);
  const record = asRecord(entry, name);
  const bytes = readFileSync(join(sourceRoot, name));
  assert.equal(bytes.length, asNumber(record.bytes), name);
  assert.equal(sha256(bytes), asString(record.sha256), name);
}
const before = readJsonRecord(
  new URL("./historical-public-0.600.0/diagnostics.stdout.txt", import.meta.url),
);
assert.equal(before.errorCount, 13, "the original four-file public baseline is immutable");
const nativeWitness = readJsonRecord(
  new URL("./source-native-10b/before-type-queries.json.txt", import.meta.url),
);
assert.ok(asString(nativeWitness.wholeGeneratedSource).length > 100);
assert.ok(asArray(nativeWitness.wholeDiagnostics).length > 0);

mkdirSync(join(consumer, "src"), { recursive: true });
for (const name of [
  "Setup.vue",
  "Loop.vue",
  "Wrong.vue",
  "mixin.d.ts",
  "Box.vue",
  "Options.vue",
  "Directive.vue",
])
  copyFileSync(join(sourceRoot, "fixtures", name + ".txt"), join(consumer, "src", name));
copyFileSync(join(sourceRoot, "fixtures/tsconfig.json.txt"), join(consumer, "tsconfig.json"));
writeFileSync(
  join(consumer, "package.json"),
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: { vize: version, vue: "3.5.42", "@typescript/typescript-linux-x64": "7.0.2" },
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
copyFileSync(identityFile, join(consumer, "release-identity.json"));
const env: NodeJS.ProcessEnv = { ...process.env };
delete env.NODE_PATH;
delete env.NAPI_RS_NATIVE_LIBRARY_PATH;
delete env.VIZE_NATIVE_PATH;
type Command = {
  label: string;
  argv: readonly string[];
  status: number | null;
  stdoutSha256: string;
  stderrSha256: string;
};
const commands: Command[] = [];
function run(label: string, argv: readonly string[], statuses: readonly number[]): string {
  assert.ok(argv.length > 0);
  const result = spawnSync(argv[0], argv.slice(1), {
    cwd: consumer,
    env,
    encoding: "utf8",
    timeout: 240_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  writeFileSync(join(consumer, label + ".stdout"), stdout);
  writeFileSync(join(consumer, label + ".stderr"), stderr);
  writeFileSync(join(consumer, label + ".status"), String(result.status) + "\n");
  commands.push({
    label,
    argv,
    status: result.status,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
  });
  writeFileSync(join(consumer, "commands.json"), JSON.stringify(commands, null, 2) + "\n");
  assert.equal(result.error, undefined, label);
  assert.ok(statuses.includes(result.status ?? -1), label + ": " + stdout + "\n" + stderr);
  return stdout;
}
run(
  "install",
  [
    "npm",
    "install",
    "--registry=https://registry.npmjs.org",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
  ],
  [0],
);
const cli = join(consumer, "node_modules/.bin/vize");
assert.equal(run("version", [cli, "--version"], [0]).trim(), "vize " + version);
for (const name of ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"])
  assert.equal(
    readJsonRecord(join(consumer, "node_modules", name, "package.json")).version,
    version,
    name,
  );
assert.equal(readJsonRecord(join(consumer, "node_modules/vue/package.json")).version, "3.5.42");
assert.equal(
  readJsonRecord(join(consumer, "node_modules/@typescript/typescript-linux-x64/package.json"))
    .version,
  "7.0.2",
);
const nativeIdentity = asRecord(
  parseJson(
    run(
      "native-identity",
      [
        process.execPath,
        "--input-type=module",
        "-e",
        'import {createRequire} from "node:module"; const require=createRequire(import.meta.url); const native=require("@vizejs/native"); const identity=require("@vizejs/native/native-binding-identity.js").getBindingIdentity(native); console.log(JSON.stringify({identity,token:native.getNativeBuildToken(),binary:require.resolve("@vizejs/native-linux-x64-gnu")}));',
      ],
      [0],
    ),
  ),
);
const loaded = asRecord(nativeIdentity.identity);
assert.equal(loaded.kind, "native");
assert.equal(loaded.token, nativeIdentity.token);
const provider = join(consumer, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
const imageManifest = [asString(nativeIdentity.binary), provider].map((file) => {
  const bytes = readFileSync(file);
  return { file, bytes: bytes.length, sha256: sha256(bytes) };
});
writeFileSync(
  join(consumer, "installed-image-manifest.json"),
  JSON.stringify(imageManifest, null, 2) + "\n",
);
const expected: Record<string, readonly string[]> = {
  "src/Setup.vue": [],
  "src/Box.vue": [],
  "src/Options.vue": [],
  "src/mixin.d.ts": [],
  "src/Directive.vue": [],
  "src/Loop.vue": [
    "error:6:23 [TS2339] Property 'missing' does not exist on the component instance.",
    "error:6:34 [TS2339] Property 'missing' does not exist on the component instance.",
  ],
  "src/Wrong.vue": [
    "error:5:14 [TS2339] Property 'toFixed' does not exist on type 'boolean'.",
    "error:6:17 [TS2339] Property 'missingStringMember' does not exist on type 'string'.",
    "error:7:9 [TS18046] 'opaque' is of type 'unknown'.",
    "error:9:23 [TS2345] Argument of type 'number' is not assignable to parameter of type 'string'.",
    "error:10:9 [TS2339] Property 'missing' does not exist on the component instance.",
    "error:10:23 [TS2339] Property 'missing' does not exist on the component instance.",
  ],
};
function check(label: string, directive: readonly string[], errorCount: number): void {
  const stdout = run(
    label,
    [
      cli,
      "check",
      "--no-config",
      "--format",
      "json",
      "--show-virtual-ts",
      "--corsa-path",
      provider,
    ],
    [1],
  );
  const packet = asRecord(parseJson(stdout));
  const files = asArray(packet.files).map((value) => asRecord(value));
  assert.deepEqual(files.map((file) => asString(file.file)).sort(), Object.keys(expected).sort());
  for (const file of files) {
    const name = asString(file.file);
    const raw = asStringArray(file.diagnostics);
    const observed = raw.map((message) => message.split(" (source:", 1)[0]);
    assert.deepEqual(
      observed,
      name === "src/Directive.vue" ? directive : expected[name],
      label + "/" + name,
    );
  }
  assert.equal(packet.errorCount, errorCount, label);
  assert.equal(packet.warningCount, 0, label);
}
const authoredDirective = readFileSync(join(consumer, "src/Directive.vue"), "utf8");
check("check-original", [], 8);
assert.ok(authoredDirective.includes("\n\t\tx\n"));
const unused = authoredDirective.replace("\n\t\tx\n", "\n\t\t1\n");
writeFileSync(join(consumer, "Directive-unused.vue.txt"), unused);
writeFileSync(join(consumer, "src/Directive.vue"), unused);
check("check-unused-directive", ["error:7:3 [TS2578] Unused '@ts-expect-error' directive."], 9);
const unprotected = authoredDirective.replace("// @ts-expect-error", "// removed directive");
assert.notEqual(unprotected, authoredDirective);
writeFileSync(join(consumer, "Directive-unprotected.vue.txt"), unprotected);
writeFileSync(join(consumer, "src/Directive.vue"), unprotected);
check(
  "check-unprotected-missing",
  ["error:8:3 [TS2339] Property 'x' does not exist on the component instance."],
  9,
);
writeFileSync(join(consumer, "src/Directive.vue"), authoredDirective);
writeFileSync(
  join(consumer, "acceptance.json"),
  JSON.stringify(
    {
      version,
      status: "pass",
      releaseIdentity: identity,
      sourceNativeWitness: "10b, not a pre10933 producer",
      nativeIdentity,
      imageManifest,
      originalErrorCount: 8,
      warningCount: 0,
      directive: { original: "clean", unused: "TS2578@7:3", unprotected: "TS2339@8:3" },
      wholePackets: [
        "check-original.stdout",
        "check-unused-directive.stdout",
        "check-unprotected-missing.stdout",
      ],
      wholeVirtualTs: [
        "check-original.stderr",
        "check-unused-directive.stderr",
        "check-unprotected-missing.stderr",
      ],
    },
    null,
    2,
  ) + "\n",
);
console.log(
  JSON.stringify({
    version,
    status: "pass",
    consumer,
    exactOriginalErrors: 8,
    directiveNegativeControls: 2,
  }),
);
