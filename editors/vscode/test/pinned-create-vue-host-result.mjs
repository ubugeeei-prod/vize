import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const expectedResult = {
  brokenDiagnostics: [
    {
      code: 2322,
      message: "Type 'string' is not assignable to type 'number'.",
      range: [1, 6, 1, 11],
      severity: 0,
      source: "vize/types",
    },
  ],
  documentDirty: true,
  extensionActive: true,
  fixtureId: "create-vue",
  repairedDiagnostics: [],
  schemaVersion: 1,
};

export function readPinnedCreateVueHostResult(resultPath) {
  assert.ok(fs.existsSync(resultPath), `packaged host did not write its result: ${resultPath}`);
  const actual = JSON.parse(fs.readFileSync(resultPath, "utf8"));
  assertServerInfo(actual.serverInfo);
  assert.deepEqual(actual, { ...expectedResult, serverInfo: actual.serverInfo });
  return actual;
}

function assertServerInfo(serverInfo) {
  assert.equal(serverInfo?.source, "configured");
  assert.equal(serverInfo?.status, "ready");
  assert.equal(typeof serverInfo?.path, "string");
  assert.ok(path.isAbsolute(serverInfo.path), "selected server path must be absolute");
  assert.ok(fs.existsSync(serverInfo.path), `selected server path must exist: ${serverInfo.path}`);
  assert.deepEqual(Object.keys(serverInfo).sort(), [
    "extensionVersion",
    "path",
    "source",
    "status",
    "version",
  ]);
  assert.equal(typeof serverInfo?.version, "string");
  assert.equal(typeof serverInfo?.extensionVersion, "string");
  assert.match(serverInfo.version, /^\d+\.\d+\.\d+(?:[-+][^\s]+)?$/);
  assert.equal(
    serverInfo.version,
    serverInfo.extensionVersion,
    "selected server version must match extension version",
  );
}
