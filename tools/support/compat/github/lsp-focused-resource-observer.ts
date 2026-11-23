import crypto from "node:crypto";
import fs from "node:fs";
import { format } from "node:util";
import { createSampler } from "../davinci/lib/lsp-resource-sampler.ts";

const [rootPid, directory] = process.argv.slice(2);
const source = "tools/support/compat/davinci/lib/lsp-resource-sampler.ts";
const sampler = createSampler(Number(rootPid));
const started = process.hrtime.bigint();
let count = 0;
const metadata = {
  sourceRevision: process.env.GITHUB_SHA,
  samplerSource: source,
  samplerSha256: crypto.createHash("sha256").update(fs.readFileSync(source)).digest("hex"),
  rootPid: Number(rootPid),
  scope:
    "workflow harness and sampled descendants, including observer; reparented CPU after leaving the tree is excluded",
  instrumented: true,
};

function record() {
  const snapshot = sampler.sample();
  const processes = snapshot.processes.map((entry) => {
    let executable = null;
    try {
      executable = fs.readlinkSync(`/proc/${entry.pid}/exe`);
    } catch {}
    return { ...entry, executable };
  });
  fs.appendFileSync(
    `${directory}/resource-samples.jsonl`,
    JSON.stringify({
      ...snapshot,
      processes,
      wallTime: new Date().toISOString(),
      elapsedNs: String(process.hrtime.bigint() - started),
    }) + "\n",
  );
  count++;
  return snapshot;
}

record();
const interval = setInterval(record, 500);
process.once("SIGTERM", () => {
  clearInterval(interval);
  let final;
  let result;
  let failure;
  try {
    final = record();
  } catch (error) {
    failure = error;
  }
  try {
    result = sampler.stop();
  } catch (error) {
    failure ??= error;
  }
  if (failure) {
    fs.writeFileSync(`${directory}/resource-error.txt`, format("%s", failure));
    process.exitCode = 1;
    return;
  }
  try {
    fs.writeFileSync(
      `${directory}/resource-summary.json`,
      JSON.stringify(
        {
          ...metadata,
          ...result,
          final,
          rawSamples: count,
        },
        null,
        2,
      ) + "\n",
    );
  } catch (error) {
    fs.writeFileSync(`${directory}/resource-error.txt`, format("%s", error));
    process.exitCode = 1;
  }
});
