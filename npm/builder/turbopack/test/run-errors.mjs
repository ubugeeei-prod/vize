import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { output } from "./prepare-consumer.mjs";

/** Verify actual host diagnostics and recovery without masking browser errors. */
export async function checkHostErrors(url, filename, original, measurements) {
  async function responseWith(status, label) {
    const start = performance.now();
    const deadline = Date.now() + 90_000;
    let packet;
    do {
      const response = await fetch(url);
      packet = { status: response.status, html: await response.text() };
      if (packet.status === status) break;
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
    const error = await responseWith(500, "invalid-sfc");
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
      const error = await responseWith(500, label);
      assert.match(error, /FixtureCounter\.vue/);
      assert.match(error, /Vapor SFC blocks are not supported/);
    } finally {
      await fs.writeFile(filename, original);
    }
    assert.match(await responseWith(200, `${label}-recovered`), /Counter 2 🐱/);
  }
}
