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
