import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const repoRoot = path.resolve(packageRoot, "../../..");

// Same artifact root as prepare-consumer.mjs; do not import that module from this probe.
function artifactRoot() {
  return path.resolve(
    repoRoot,
    process.env.VIZE_TURBOPACK_ARTIFACTS ??
      `target/turbopack-consumers/${process.env.GITHUB_RUN_ID ?? "local"}-${process.env.GITHUB_RUN_ATTEMPT ?? "1"}`,
  );
}

/** Verify actual host diagnostics and recovery without masking browser errors. */
export async function checkHostErrors(url, filename, original, measurements) {
  const output = artifactRoot();
  async function responseWith(status, label, diagnostic) {
    const start = performance.now();
    const deadline = Date.now() + 90_000;
    let packet;
    do {
      const response = await fetch(url);
      packet = { status: response.status, html: await response.text() };
      // A framework 500 can name the missing module before the Vize diagnostic is ready.
      const ready =
        packet.status === status && (diagnostic === undefined || diagnostic.test(packet.html));
      if (ready) break;
      await new Promise((resolve) => setTimeout(resolve, 100));
    } while (Date.now() < deadline);
    await fs.writeFile(path.join(output, `${label}.html`), packet.html);
    await fs.writeFile(
      path.join(output, `${label}.json`),
      JSON.stringify({ status: packet.status }) + "\n",
    );
    measurements.push({ phase: label, ms: performance.now() - start });
    assert.equal(packet.status, status);
    return packet.html;
  }
  try {
    assert.ok(original.includes("</section>"));
    await fs.writeFile(filename, original.replace("</section>", "</div>"));
    const error = await responseWith(500, "invalid-sfc", /\[vize\]/);
    assert.match(error, /FixtureCounter\.vue/);
    assert.match(error, /\[vize\]/);
  } finally {
    await fs.writeFile(filename, original);
  }
  assert.match(await responseWith(200, "recovered-sfc"), /Counter 2 🐱/);
  for (const [label, marker] of [
    ["vapor-presence", "vapor"],
    ["vapor-false-value", 'vapor="false"'],
  ]) {
    try {
      assert.ok(original.includes("<script setup"));
      await fs.writeFile(filename, original.replace("<script setup", `<script setup ${marker}`));
      const error = await responseWith(500, label, /Vapor SFC blocks are not supported/);
      assert.match(error, /FixtureCounter\.vue/);
      assert.match(error, /Vapor SFC blocks are not supported/);
    } finally {
      await fs.writeFile(filename, original);
    }
    assert.match(await responseWith(200, `${label}-recovered`), /Counter 2 🐱/);
  }
}
