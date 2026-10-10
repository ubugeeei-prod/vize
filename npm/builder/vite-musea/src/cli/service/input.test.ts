import assert from "node:assert/strict";
import { createServer, request } from "node:http";
import test from "node:test";
import { captureInput } from "./auth.ts";

void test("a capture Art path preserves authored UTF-8 across actual HTTP chunks", async () => {
  const authored = { artPath: "src/ボタン😀.art.vue", update: false };
  const body = Buffer.from(JSON.stringify(authored));
  const cut = body.indexOf(Buffer.from("ボ")) + 1;
  assert.ok(cut > 0);
  let observed!: () => void;
  const firstChunk = new Promise<void>((resolve) => {
    observed = resolve;
  });
  let chunks = 0;
  const server = createServer((incoming, response) => {
    incoming.on("data", () => {
      chunks++;
      observed();
    });
    void captureInput(incoming).then(
      (input) => {
        response.setHeader("Content-Type", "application/json");
        response.end(JSON.stringify(input));
      },
      (error) => {
        response.writeHead(400).end(String(error));
      },
    );
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  try {
    const client = request(`http://127.0.0.1:${address.port}/capture`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });
    const result = new Promise<unknown>((resolve, reject) => {
      client.once("error", reject);
      client.once("response", (response) => {
        let bytes = "";
        response.on("data", (chunk) => {
          bytes += String(chunk);
        });
        response.once("end", () => {
          assert.equal(response.statusCode, 200);
          resolve(JSON.parse(bytes));
        });
      });
    });
    client.write(body.subarray(0, cut));
    await firstChunk;
    await new Promise<void>((resolve) => setImmediate(resolve));
    client.end(body.subarray(cut));
    assert.deepEqual(await result, authored);
    assert.ok(chunks >= 2);
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});

void test("capture HTTP media types accept JSON parameters and preserve bounded refusal messages", async () => {
  const authored = { artPath: "src/ボタン😀.art.vue", update: false };
  const server = createServer((incoming, response) => {
    void captureInput(incoming).then(
      (input) => response.end(JSON.stringify(input)),
      (error) => response.writeHead(400).end(String(error)),
    );
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  try {
    const url = `http://127.0.0.1:${address.port}/capture`;
    for (const type of [
      "application/json",
      "application/json; charset=utf-8",
      "Application/JSON; Charset=UTF-8",
    ]) {
      const actual = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": type },
        body: JSON.stringify(authored),
      });
      assert.equal(actual.status, 200, type);
      assert.deepEqual(await actual.json(), authored);
    }
    for (const [type, body, message] of [
      ["text/plain", JSON.stringify(authored), "Expected JSON"],
      ["application/json", "{", "Capture input must contain valid JSON"],
      [
        "application/json",
        JSON.stringify({ ...authored, update: "yes" }),
        "Capture requires a known artPath and boolean update",
      ],
      [
        "application/json",
        JSON.stringify({ ...authored, extra: true }),
        "Capture requires a known artPath and boolean update",
      ],
      ["application/json", " ".repeat(4097), "Capture input is too large"],
    ]) {
      const actual = await fetch(url, { method: "POST", headers: { "Content-Type": type }, body });
      assert.equal(actual.status, 400);
      assert.equal(await actual.text(), `Error: ${message}`);
    }
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
