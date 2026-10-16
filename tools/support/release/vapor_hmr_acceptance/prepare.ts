import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publicInputs } from "../public_acceptance/inputs.ts";
import { rejectOverrides } from "../public_acceptance/installed.ts";
import { derivePublicationPlan, readRawBlob } from "../public_acceptance/plan.ts";
import { verifyPublication } from "../public_acceptance/registry.ts";
import { sources, vueVersion } from "./fixtures.ts";

rejectOverrides();
const baseline = process.argv[2] === "baseline";
assert.ok(process.argv.length === (baseline ? 3 : 2));
const root = fs.realpathSync(process.cwd());
const output = path.resolve(process.env.PUBLIC_OUTPUT!);
fs.mkdirSync(output, { recursive: true });
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
let version = "0.457.0";
let publication: unknown;
let source: unknown;
if (!baseline) {
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  const inputs = publicInputs(process.env);
  const plan = derivePublicationPlan(root, inputs.head);
  // A release predating the actual repair cannot qualify #9180.
  execFileSync(
    "git",
    [
      "--no-replace-objects",
      "merge-base",
      "--is-ancestor",
      "cd861aa001f7ea458f8b0147e5b9c850b9db1c87",
      inputs.cut,
    ],
    { cwd: root },
  );
  publication = await verifyPublication(plan, inputs);
  version = plan.version;
  source = inputs;
}
const consumer = fs.mkdtempSync(
  path.join(fs.realpathSync(process.env.RUNNER_TEMP!), "vize-vapor-hmr-"),
);
assert.ok(!consumer.startsWith(root + path.sep));
const write = (file: string, bytes: Buffer | string) => {
  fs.mkdirSync(path.dirname(path.join(consumer, file)), { recursive: true });
  fs.writeFileSync(path.join(consumer, file), bytes, { flag: "wx" });
  return { file, sha256: digest(bytes) };
};
const fixtures = Object.entries(sources).map(([file, bytes]) => write(file, bytes));
const history = "tests/_fixtures/differential/compiler/vapor-hmr-9180/historical-report.md";
write(
  "historical-report.md",
  readRawBlob(
    root,
    baseline
      ? execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
          encoding: "utf8",
        }).trim()
      : (source as { head: string }).head,
    history,
  ),
);
const dependencies = Object.fromEntries(
  [
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
    "@vizejs/unplugin",
  ].map((name) => [name, version]),
);
write(
  "package.json",
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: {
        ...dependencies,
        vue: vueVersion,
        "vue-router": "4.5.1",
        vite: "8.3.2",
        playwright: "1.62.1",
      },
    },
    null,
    2,
  ) + "\n",
);
const directory = fileURLToPath(new URL("./", import.meta.url));
const harness = [
  write("vapor_hmr_acceptance/runtime.ts", fs.readFileSync(path.join(directory, "runtime.ts"))),
  write(
    "public_acceptance/installed.ts",
    fs.readFileSync(path.join(directory, "../public_acceptance/installed.ts")),
  ),
];
for (const [file, value] of Object.entries({
  "preparation.json": { consumer, version, vueVersion, baseline, source, fixtures, harness },
  "publication.json": publication ?? { baseline: true },
})) {
  fs.writeFileSync(path.join(output, file), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
assert.ok(!/[\r\n]/u.test(consumer));
fs.appendFileSync(process.env.GITHUB_ENV!, `PUBLIC_HMR_CONSUMER=${consumer}\n`);
console.log(JSON.stringify({ consumer, version, baseline, vueVersion }));
