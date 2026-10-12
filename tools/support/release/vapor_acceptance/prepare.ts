import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publicInputs } from "../public_acceptance/inputs.ts";
import { rejectOverrides } from "../public_acceptance/installed.ts";
import { derivePublicationPlan, readRawBlob } from "../public_acceptance/plan.ts";
import { verifyPublication } from "../public_acceptance/registry.ts";
import { digest, materializeFixtures, reportedVue } from "./fixtures.ts";

rejectOverrides();
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
const root = fs.realpathSync(process.cwd());
const source = publicInputs(process.env);
const plan = derivePublicationPlan(root, source.head);
const publication = await verifyPublication(plan, source);
const output = path.resolve(process.env.PUBLIC_OUTPUT!);
fs.mkdirSync(output, { recursive: true });
const consumer = fs.mkdtempSync(
  path.join(fs.realpathSync(process.env.RUNNER_TEMP!), "vize-public-vapor-"),
);
assert.ok(!consumer.startsWith(root + path.sep));
const fixtures = materializeFixtures(consumer, (file) => readRawBlob(root, source.head, file));
const dependencies = Object.fromEntries(
  [
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
    "@vizejs/unplugin",
  ].map((name) => [name, plan.version]),
);
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: {
        ...dependencies,
        vue: reportedVue,
        "@vue/compiler-sfc": reportedVue,
        vite: "8.3.2",
        playwright: "1.62.1",
      },
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
const directory = fileURLToPath(new URL("./", import.meta.url));
const harness = [
  ["vapor_acceptance/runtime.ts", path.join(directory, "runtime.ts")],
  ["vapor_acceptance/fixtures.ts", path.join(directory, "fixtures.ts")],
  ["public_acceptance/installed.ts", path.join(directory, "../public_acceptance/installed.ts")],
].map(([file, origin]) => {
  const bytes = fs.readFileSync(origin);
  fs.mkdirSync(path.dirname(path.join(consumer, file)), { recursive: true });
  fs.writeFileSync(path.join(consumer, file), bytes, { flag: "wx" });
  return { file, sha256: digest(bytes) };
});
const save = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
save("publication.json", publication);
save("preparation.json", {
  source,
  consumer,
  version: plan.version,
  reportedVue,
  fixtures,
  harness,
  observer: execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim(),
  manifestSha256: digest(fs.readFileSync(path.join(consumer, "package.json"))),
});
assert.ok(!/[\r\n]/u.test(consumer));
fs.appendFileSync(
  process.env.GITHUB_ENV!,
  `PUBLIC_VAPOR_CONSUMER=${consumer}\nPUBLIC_VERSION=${plan.version}\n`,
);
console.log(JSON.stringify({ source, reportedVue, original: 6, authored: 2, prepared: true }));
