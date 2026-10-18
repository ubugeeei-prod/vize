import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { nativeHistorySource, nativeHistoryDirectory } from "./formatter-history-build.mts";

export const ciRequestPath = (nativeDir) =>
  path.join(nativeHistoryDirectory(nativeDir), "ci-build-request.json");
const actualContext = () => ({
  platform: process.platform,
  githubActions: process.env.GITHUB_ACTIONS,
  runId: process.env.GITHUB_RUN_ID,
});
export function validateCiRequest(request, source, context = actualContext()) {
  assert.equal(context.platform, "linux");
  assert.equal(context.githubActions, "true");
  assert.match(context.runId ?? "", /^[1-9][0-9]*$/);
  assert.deepEqual(Object.keys(request), ["schema", "version", "source", "runId"]);
  assert.equal(request.schema, "vize.public-native-formatter-ci-build-request");
  assert.equal(request.version, 1);
  assert.deepEqual(request.source, { head: source.head, tree: source.tree });
  assert.equal(request.runId, context.runId);
  return request;
}
export function readCiRequest(nativeDir, source) {
  const file = ciRequestPath(nativeDir);
  if (!fs.existsSync(file)) return null;
  return validateCiRequest(JSON.parse(fs.readFileSync(file, "utf8")), source);
}
export function writeCiRequest(nativeDir) {
  const source = nativeHistorySource(nativeDir);
  const request = {
    schema: "vize.public-native-formatter-ci-build-request",
    version: 1,
    source: { head: source.head, tree: source.tree },
    runId: process.env.GITHUB_RUN_ID,
  };
  validateCiRequest(request, source);
  fs.mkdirSync(nativeHistoryDirectory(nativeDir), { recursive: true });
  fs.writeFileSync(ciRequestPath(nativeDir), JSON.stringify(request) + "\n", { flag: "wx" });
  console.log("Requested one source-bound CI capture; no build or qualification credit.");
}
