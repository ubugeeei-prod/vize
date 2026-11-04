import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export async function waitForDevelopmentSsr(options: {
  url: string;
  output: string;
  running: () => boolean;
  timeoutMs?: number;
}): Promise<string> {
  const deadline = Date.now() + (options.timeoutMs ?? 30_000);
  const startup = path.join(options.output, "startup");
  fs.mkdirSync(startup);
  let attempt = 0;
  while (Date.now() < deadline) {
    assert.ok(options.running(), "Nuxt development server stopped before becoming ready");
    const prefix = path.join(startup, String(++attempt).padStart(4, "0"));
    let response: Response;
    try {
      response = await fetch(options.url, {
        signal: AbortSignal.timeout(Math.max(1, Math.min(2_000, deadline - Date.now()))),
      });
    } catch (error) {
      fs.writeFileSync(prefix + ".json", JSON.stringify({ error: String(error) }) + "\n");
      if (!options.running()) throw error;
      await new Promise((resolve) => setTimeout(resolve, 100));
      continue;
    }
    const html = await response.text();
    const observation =
      JSON.stringify(
        {
          status: response.status,
          headers: Object.fromEntries(response.headers),
        },
        null,
        2,
      ) + "\n";
    fs.writeFileSync(prefix + ".json", observation);
    fs.writeFileSync(prefix + ".html", html);
    if (response.status === 503) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      continue;
    }
    fs.writeFileSync(path.join(options.output, "ssr-response.json"), observation);
    fs.writeFileSync(path.join(options.output, "ssr.html"), html);
    assert.equal(response.status, 200, "Nuxt development SSR returned a rendering error");
    return html;
  }
  assert.fail("Nuxt development server did not become ready");
}
