import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage } from "node:http";
import os from "node:os";
import path from "node:path";

import {
  HttpError,
  resolveInside,
  resolveUrlPathInside,
  serializeScriptValue,
  validateDevApiRequest,
} from "./security.ts";

function request(method: string, headers: IncomingMessage["headers"]): IncomingMessage {
  return { method, headers } as IncomingMessage;
}

void test("resolveInside keeps filesystem reads under the allowed directory", () => {
  const root = path.resolve("/tmp/musea-root");

  assert.equal(resolveInside(root, "src/Button.vue"), path.join(root, "src/Button.vue"));
  assert.throws(() => resolveInside(root, "../outside.txt"), HttpError);
  assert.throws(() => resolveUrlPathInside(root, "/assets/../../outside.txt"), HttpError);
});

void test("resolveInside follows links before accepting a path", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-path-"));
  const root = path.join(tempDir, "root");
  const outside = path.join(tempDir, "outside");

  try {
    await fs.promises.mkdir(root);
    await fs.promises.mkdir(outside);
    await fs.promises.symlink(outside, path.join(root, "linked"), "dir");

    assert.throws(() => resolveInside(root, "linked/file.txt"), HttpError);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("validateDevApiRequest requires same-origin JSON mutations with the session token", () => {
  const token = "session-token";

  assert.equal(
    validateDevApiRequest(
      request("PUT", {
        host: "localhost:5173",
        origin: "http://localhost:5173",
        "content-type": "application/json",
        "x-musea-session": token,
      }),
      token,
    ),
    null,
  );

  assert.equal(
    validateDevApiRequest(
      request("PUT", {
        host: "localhost:5173",
        origin: "http://evil.test",
        "content-type": "application/json",
        "x-musea-session": token,
      }),
      token,
    )?.status,
    403,
  );

  assert.equal(
    validateDevApiRequest(
      request("POST", {
        host: "localhost:5173",
        origin: "http://localhost:5173",
        "content-type": "text/plain",
        "x-musea-session": token,
      }),
      token,
    )?.status,
    415,
  );

  assert.equal(
    validateDevApiRequest(
      request("DELETE", {
        host: "localhost:5173",
        origin: "http://localhost:5173",
        "content-type": "application/json",
      }),
      token,
    )?.status,
    403,
  );
});

void test("serializeScriptValue cannot close the surrounding script tag", () => {
  assert.equal(
    serializeScriptValue("</script><script>alert(1)</script>"),
    `"\\u003C/script\\u003E\\u003Cscript\\u003Ealert(1)\\u003C/script\\u003E"`,
  );
});
