import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runHostProcesses } from "./ci-host-concurrency.mjs";
import { publishReplayFailure } from "./n8n-preparation-overlap.mjs";
import {
  preparationFixture,
  read,
  until,
} from "../../../tests/tooling/support/oxlint-preparation-fixture.mjs";

void test("the actual host worker runs all original preparation commands before waiting and refuses failed baseline before n8n", async (t) => {
  const f = preparationFixture(t);
  const packageDir = path.join(f.artifacts, "controlled-package");
  const temporary = path.join(f.artifacts, "controlled-installed-host");
  const output = path.join(f.artifacts, "1.78.0"),
    bin = path.join(f.artifacts, "bin");
  for (const directory of [path.join(packageDir, "src"), temporary, output, bin])
    fs.mkdirSync(directory, { recursive: true });
  const npm = path.join(bin, "npm");
  fs.writeFileSync(
    npm,
    `#!${process.execPath}
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
assert.deepEqual(process.argv.slice(2),["install","--ignore-scripts","--no-audit","--no-fund","--legacy-peer-deps","oxlint@1.78.0","oxlint-tsgolint@7.0.2001"]);
for (const [name,version] of [["oxlint","1.78.0"],["oxlint-tsgolint","7.0.2001"]]) {
 fs.mkdirSync(path.join("node_modules",name,"bin"),{recursive:true});
 fs.writeFileSync(path.join("node_modules",name,"package.json"),JSON.stringify({version}));
}
fs.writeFileSync("package-lock.json","complete original controlled installation lock\\r\\n");
console.log("complete install raw output");
`,
  );
  fs.chmodSync(npm, 0o755);
  // These isolated child fixtures prove the production worker's command and
  // scheduling boundaries only; no native implementation is substituted or qualified.
  fs.writeFileSync(
    path.join(packageDir, "src/project-transport.test.ts"),
    `
import fs from "node:fs";
import { createHash } from "node:crypto";
const custody=JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY));
const common={source:custody.source,binary:custody.binary.path,binarySha256:custody.binary.sha256};
const source="controlled verifier input",sha=createHash("sha256").update(source).digest("hex");
const rows=[...Array.from({length:5},(_,i)=>({...common,kind:"load",pid:i+1,control:"Guard-"+i+".vue"})),
 {...common,kind:"load",pid:10},...Array.from({length:18},()=>({...common,kind:"call",pid:10,outcome:"return",hasRuleHint:true,args:[source,{filename:"/Controlled.vue",enabledRules:Array(51).fill("control-rule")}],originalSha256:sha}))];
fs.writeFileSync(custody.calls,rows.map(x=>JSON.stringify(x)).join("\\n")+"\\n");
fs.writeFileSync(process.env.VIZE_OXLINT_PROJECT_CAPTURE,JSON.stringify({qualified:{imports:true,typeAware:true}}));
console.log("complete transport raw output");
`,
  );
  for (const file of ["script-safe-transport.test.ts", "original-ignore.test.ts"])
    fs.writeFileSync(path.join(packageDir, "src", file), "// Controlled command membership.\n");
  const marker = path.join(f.artifacts, "html-preparation.done");
  fs.writeFileSync(
    path.join(packageDir, "src/html-cli.test.mjs"),
    `
import fs from "node:fs";
fs.writeFileSync(process.env.VIZE_OXLINT_HTML_CAPTURE,JSON.stringify({complete:true,qualified:{controlled:true}}));
fs.writeFileSync(${JSON.stringify(marker)},JSON.stringify({owner:process.ppid}));
console.log("complete HTML raw output");
`,
  );
  const configuration = path.join(output, "worker-config.json");
  fs.writeFileSync(
    configuration,
    JSON.stringify({
      packageDir,
      artifacts: f.artifacts,
      binary: f.binary,
      version: "1.78.0",
      types: "7.0.2001",
      temporary,
    }),
  );
  const environment = {
    ...process.env,
    GITHUB_ACTIONS: "true",
    GITHUB_SHA: f.receipt.source.head,
    PATH: `${bin}${path.delimiter}${process.env.PATH}`,
    TMPDIR: temporary,
    TMP: temporary,
    TEMP: temporary,
  };
  delete environment.NODE_TEST_CONTEXT;
  const jobs = [
    {
      id: "1.78.0",
      output,
      command: process.execPath,
      args: [fileURLToPath(new URL("./ci-host-worker.mjs", import.meta.url)), configuration],
      cwd: packageDir,
      timeoutMs: 6000,
      env: environment,
    },
  ];
  const terminal = runHostProcesses(jobs, { workers: 1 });
  let completed;
  void terminal.then((results) => {
    completed = results;
  });
  await until(() => fs.existsSync(marker) || completed);
  assert.ok(
    fs.existsSync(marker),
    `host ended before baseline wait: ${JSON.stringify(completed)}\n${fs.readFileSync(path.join(output, "worker.stderr.log"), "utf8")}`,
  );
  await new Promise((resolve) => setTimeout(resolve, 100));
  assert.doesNotThrow(
    () => process.kill(read(marker).owner, 0),
    "actual worker must still await baseline",
  );
  assert.ok(!fs.existsSync(path.join(output, "phase-walltime.json")));
  assert.ok(!fs.existsSync(path.join(output, "host-worker-result.json")));
  for (const file of [
    "install.json",
    "tests.json",
    "html-tests.json",
    "original-project.json",
    "original-html-cli.json",
    "package-lock.json",
  ])
    assert.ok(fs.existsSync(path.join(output, file)), file);
  publishReplayFailure(f);
  const results = await terminal;
  assert.equal(results[0].status, 1);
  const phases = read(path.join(output, "phase-walltime.json"));
  assert.equal(phases.status, "failed");
  for (const phase of ["install", "transport", "htmlTransport", "baselineWait"])
    assert.ok(phases.phaseWalltimeMs[phase] >= 0);
  assert.equal(phases.phaseWalltimeMs.n8nReplay, undefined);
  assert.ok(!fs.existsSync(temporary), "actual worker owns cleanup after refusal");
  assert.equal(read(path.join(output, "html-tests.json")).stdout, "complete HTML raw output\n");
  assert.match(
    fs.readFileSync(path.join(output, "worker.stderr.log"), "utf8"),
    /baseline\/reference preparation failed/u,
  );
});
