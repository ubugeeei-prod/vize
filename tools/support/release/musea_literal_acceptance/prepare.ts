import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publicInputs } from "../public_acceptance/inputs.ts";
import { rejectOverrides } from "../public_acceptance/installed.ts";
import { derivePublicationPlan, readRawBlob } from "../public_acceptance/plan.ts";
import { publicReader } from "../public_acceptance/http.ts";
import { verifyGithub } from "../public_acceptance/github.ts";
import { digest, fixtureRoots, materialize } from "./fixtures.ts";

rejectOverrides();
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
const root = fs.realpathSync(process.cwd()),
  source = publicInputs(process.env);
const plan = derivePublicationPlan(root, source.head);
assert.equal(plan.parentCut, source.cut);
assert.equal(source.tag, `v${plan.version}`);
const publication = await verifyGithub(source, publicReader().json);
const output = path.resolve(process.env.PUBLIC_OUTPUT!);
fs.mkdirSync(output, { recursive: true });
const consumer = fs.mkdtempSync(
  path.join(fs.realpathSync(process.env.RUNNER_TEMP!), "vize-public-musea-literal-"),
);
assert.ok(!consumer.startsWith(root + path.sep));
const files = execFileSync(
  "git",
  ["--no-replace-objects", "ls-tree", "-r", "--name-only", source.head, "--", ...fixtureRoots],
  { encoding: "utf8" },
)
  .trim()
  .split("\n");
const fixtures = materialize(consumer, files, (file) => readRawBlob(root, source.head, file));
const dependencies = Object.fromEntries(
  [
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
  ].map((name) => {
    assert.ok(plan.npm.some((target) => target.name === name && target.version === plan.version));
    return [name, plan.version];
  }),
);
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: {
        ...dependencies,
        vite: "8.0.0",
        vue: "3.5.35",
        "vue-router": "4.5.1",
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
  ...["runtime", "suite", "fixtures", "observations"].map((name) => [
    `musea_literal_acceptance/${name}.ts`,
    path.join(directory, `${name}.ts`),
  ]),
  ["public_acceptance/installed.ts", path.join(directory, "../public_acceptance/installed.ts")],
  [
    "musea_static_acceptance/custody.ts",
    path.join(directory, "../musea_static_acceptance/custody.ts"),
  ],
].map(([file, origin]) => {
  const bytes = fs.readFileSync(origin);
  fs.mkdirSync(path.dirname(path.join(consumer, file)), { recursive: true });
  fs.writeFileSync(path.join(consumer, file), bytes, { flag: "wx" });
  return { file, sha256: digest(bytes) };
});
for (const [file, value] of Object.entries({
  "publication.json": { inputs: source, plan, ...publication },
  "preparation.json": {
    source,
    consumer,
    version: plan.version,
    fixtures,
    harness,
    observer: execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim(),
    manifestSha256: digest(fs.readFileSync(path.join(consumer, "package.json"))),
  },
}))
  fs.writeFileSync(path.join(output, file), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
assert.ok(!/[\r\n]/u.test(consumer));
fs.appendFileSync(
  process.env.GITHUB_ENV!,
  `PUBLIC_MUSEA_LITERAL_CONSUMER=${consumer}\nPUBLIC_VERSION=${plan.version}\n`,
);
console.log(
  JSON.stringify({ source, prepared: true, scope: "Musea literal names and dev VRT only" }),
);
