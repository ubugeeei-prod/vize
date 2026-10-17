import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

void test(
  "prepared check environments reach the full check and all three injected passes",
  {
    skip: process.platform === "win32",
  },
  (t) => {
    const root = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-check-provider-env-")),
    );
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.mkdirSync(path.join(root, "src"));
    const source = "<template><div /></template>\n";
    fs.writeFileSync(path.join(root, "src/Original.vue"), source);
    fs.writeFileSync(path.join(root, "tsconfig.json"), '{"compilerOptions":{"strict":true}}\n');
    const log = path.join(root, "invocations.jsonl");
    const cli = path.join(root, "checker.ts");
    fs.writeFileSync(
      cli,
      `#!${process.execPath}
import assert from 'node:assert/strict';
import fs from 'node:fs';
const args = process.argv.slice(2);
assert.equal(process.env.npm_package_name, 'checked-upstream');
assert.equal(process.env.npm_package_version, '1.2.3');
assert.equal(args[0], 'check');
assert.equal(args.at(-2), '--tsconfig');
fs.appendFileSync(process.env.INVOCATIONS, JSON.stringify({args, cwd: process.cwd()}) + '\\n');
if (process.env.PROVIDER_FAILURE) { console.error('authored config provider failed'); process.exit(2); }
const file = args[1].includes('*') ? 'src/Original.vue' : args[1];
const broken = fs.readFileSync(file, 'utf8').includes('number = "not-a-number"');
const diagnostics = broken ? ["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."] : [];
console.log(JSON.stringify({fileCount: 1, errorCount: diagnostics.length, files: [{file, diagnostics}]}));
process.exitCode = broken ? 1 : 0;
`,
      { mode: 0o755 },
    );
    const helper = new URL("../snapshots/_helpers/realworld.ts", import.meta.url).href;
    const driver = `
import assert from 'node:assert/strict';
import { runCrashFreeVizeCheck, runVizeCheckWithInjectedTypeError } from ${JSON.stringify(helper)};
const app = { name: 'upstream', check: { cwd: process.cwd(), patterns: ['src/**/*.vue'], tsconfig: 'tsconfig.json' } };
const env = { ...process.env, npm_package_name: 'checked-upstream', npm_package_version: '1.2.3' };
assert.equal(runCrashFreeVizeCheck(app, { env }).fileCount, 1);
const result = runVizeCheckWithInjectedTypeError(app, { env, tsconfig: { relativePath: 'injected.json', content: '{"extends":"./tsconfig.json"}' } });
assert.deepEqual(result.repairedDiagnostics, []);
assert.equal(result.errorCount, 1);
assert.throws(() => runCrashFreeVizeCheck(app, { env: { ...env, PROVIDER_FAILURE: '1' } }), /exit 2.*authored config provider failed/s);
`;
    const run = spawnSync(process.execPath, ["--input-type=module", "-e", driver], {
      cwd: root,
      env: { ...process.env, VIZE_BIN: cli, CORSA_BIN: "/retained/corsa", INVOCATIONS: log },
      encoding: "utf8",
    });
    assert.equal(run.status, 0, run.stdout + run.stderr);
    const calls = fs
      .readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.equal(calls.length, 5);
    assert.ok(calls.every((call) => call.cwd === root));
    assert.deepEqual(
      calls.map((call) => call.args.at(-1)),
      ["tsconfig.json", "injected.json", "injected.json", "injected.json", "tsconfig.json"],
    );
    assert.deepEqual(
      calls.slice(1, 4).map((call) => call.args[1]),
      [
        "src/__vize_intentional_type_error_upstream__.vue",
        "src/__vize_intentional_type_error_upstream__.vue",
        "src/__vize_intentional_type_error_upstream__.vue",
      ],
    );
    assert.equal(fs.readFileSync(path.join(root, "src/Original.vue"), "utf8"), source);
    assert.equal(fs.existsSync(path.join(root, "injected.json")), false);
    assert.equal(
      fs.existsSync(path.join(root, "src/__vize_intentional_type_error_upstream__.vue")),
      false,
    );
  },
);
