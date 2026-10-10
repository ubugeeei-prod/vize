import assert from "node:assert/strict";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { storedCaptureBytes } from "./hosted-vrt-controls.fixtures.ts";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";

/** Corrupt the actual owned report, then inspect the real service error and authored pane. */
export async function proveOwnedHostedError(
  page: Page,
  endpoint: string,
  local: string,
  output: string,
) {
  const report = path.join(local, "reports/vrt-Button-report.json");
  const original = await readFile(report);
  await mkdir(path.dirname(report), { recursive: true });
  const ambiguous = Buffer.from('{"reportOwner":{"version":1,"artIdentity":"foreign.art.vue"}}');
  await writeFile(report, ambiguous);
  const before = await storedCaptureBytes(local);
  try {
    const [response] = await Promise.all([
      page.waitForResponse(
        (item) => item.url() === `${endpoint}/capture` && item.request().method() === "POST",
      ),
      page.getByRole("button", { name: "Run VRT", exact: true }).click(),
    ]);
    const body = await response.body();
    const data = JSON.parse(body.toString()) as { error: string; success?: boolean };
    await writeFile(path.join(output, "actual-owned-error.json"), body);
    assert.equal(response.status(), 400);
    assert.notEqual(data.success, true);
    assert.match(data.error, /Ambiguous VRT report ownership:/);
    assert.match(data.error, /Archive or move the existing reports before retrying\./);
    await page.locator(".vrt-error").getByText(data.error, { exact: false }).waitFor();
    assert.equal(await page.locator(".vrt-error .vrt-hint").count(), 0);
    assert.deepEqual(await storedCaptureBytes(local), before);
    assert.equal(sha256(await readFile(report)), sha256(ambiguous));
    return {
      status: response.status(),
      error: data.error,
      bytes: body.length,
      sha256: sha256(body),
    };
  } finally {
    await rm(report);
    await writeFile(report, original);
  }
}
