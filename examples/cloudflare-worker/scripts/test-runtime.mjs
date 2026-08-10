import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import net from "node:net";

async function availablePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address === "object");
  await new Promise((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
  return address.port;
}

async function waitForWorker(url, child, output) {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`Wrangler exited before becoming ready.\n${output()}`);
    }
    try {
      const response = await fetch(url);
      if (response.ok) {
        return response;
      }
    } catch {
      // The local socket is expected to reject until workerd is ready.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Timed out waiting for Wrangler.\n${output()}`);
}

async function stop(child) {
  if (child.exitCode !== null) return;
  child.kill("SIGTERM");
  await Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    new Promise((resolve) => setTimeout(resolve, 5_000)),
  ]);
  if (child.exitCode === null) child.kill("SIGKILL");
}

const port = await availablePort();
const url = `http://127.0.0.1:${port}/`;
const wrangler = process.platform === "win32" ? "wrangler.cmd" : "wrangler";
const child = spawn(wrangler, ["dev", "--local", "--port", String(port)], {
  env: { ...process.env, NO_COLOR: "1" },
  stdio: ["ignore", "pipe", "pipe"],
});
let logs = "";
child.stdout.on("data", (chunk) => {
  logs += chunk;
});
child.stderr.on("data", (chunk) => {
  logs += chunk;
});

try {
  const demoResponse = await waitForWorker(url, child, () => logs);
  const demo = await demoResponse.json();
  assert.equal(demo.ok, true);
  assert.equal(demo.package, "@vizejs/wasm");
  assert.deepEqual(demo.result.errors, []);

  const response = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      source:
        '<template><main>{{ message }}</main></template><script setup lang="ts">const message: string = "Hello from workerd"</script>',
      options: { filename: "App.vue" },
    }),
  });
  const payload = await response.json();
  assert.equal(response.status, 200);
  assert.equal(payload.ok, true);
  assert.deepEqual(payload.result.errors, []);
  assert.match(payload.result.script.code, /Hello from workerd/);
  console.log("Cloudflare Worker runtime smoke passed");
} finally {
  await stop(child);
}
