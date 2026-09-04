import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readPinnedCreateVueHostResult } from "../../editors/vscode/test/pinned-create-vue-host-result.mjs";

test("packaged host result proves the pinned create-vue diagnostic transition", () => {
  const resultDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-create-vue-host-result-"));
  const resultPath = path.join(resultDirectory, "result.json");
  const serverPath = path.join(resultDirectory, "vize");
  const passingResult = createPassingResult(serverPath);

  try {
    fs.writeFileSync(serverPath, "");
    fs.writeFileSync(resultPath, JSON.stringify(passingResult));
    assert.deepEqual(readPinnedCreateVueHostResult(resultPath), passingResult);

    fs.writeFileSync(
      resultPath,
      JSON.stringify({ ...passingResult, documentDirty: false, extensionActive: false }),
    );
    assert.throws(
      () => readPinnedCreateVueHostResult(resultPath),
      /Expected values to be strictly/,
    );

    fs.writeFileSync(
      resultPath,
      JSON.stringify({
        ...passingResult,
        brokenDiagnostics: [{ ...passingResult.brokenDiagnostics[0], range: [1, 7, 1, 12] }],
      }),
    );
    assert.throws(
      () => readPinnedCreateVueHostResult(resultPath),
      /Expected values to be strictly/,
    );

    fs.writeFileSync(
      resultPath,
      JSON.stringify({
        ...passingResult,
        serverInfo: { ...passingResult.serverInfo, version: "0.391.0" },
      }),
    );
    assert.throws(
      () => readPinnedCreateVueHostResult(resultPath),
      /selected server version must match extension version/,
    );
  } finally {
    fs.rmSync(resultDirectory, { force: true, recursive: true });
  }
});

function createPassingResult(serverPath: string) {
  return {
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
    serverInfo: {
      extensionVersion: "0.392.0",
      path: serverPath,
      source: "configured",
      status: "ready",
      version: "0.392.0",
    },
  };
}
