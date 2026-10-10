import assert from "node:assert/strict";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";
import { SNAPSHOT_INDEX_FILE } from "./vrt/snapshot-index.ts";

export async function storedCaptureBytes(directory: string): Promise<Record<string, string>> {
  const files: Record<string, string> = {};
  async function visit(folder: string) {
    let entries;
    try {
      entries = await readdir(folder, { withFileTypes: true });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
      throw error;
    }
    for (const entry of entries) {
      const file = path.join(folder, entry.name);
      if (entry.isDirectory()) await visit(file);
      else if (/\.(png|html|json)$/.test(file) && entry.name !== SNAPSHOT_INDEX_FILE)
        files[path.relative(directory, file)] = sha256(await readFile(file));
    }
  }
  await visit(directory);
  return files;
}

/** Actual HTTP inputs; receipts contain statuses/bytes only, never the bearer. */
export async function refuseSessionInputs(
  session: { endpoint: string; token: string },
  origin: string,
  artPath: string,
  directory: string,
) {
  const initial = await storedCaptureBytes(directory);
  const cases = [
    { name: "missing-token", origin, token: "", status: 401 },
    { name: "wrong-token", origin, token: "wrong", status: 401 },
    {
      name: "foreign-origin",
      origin: "https://foreign.example",
      token: session.token,
      status: 403,
    },
    { name: "null-origin", origin: "null", token: session.token, status: 403 },
    { name: "missing-origin", origin: "", token: session.token, status: 403 },
  ];
  const receipts: { name: string; status: number }[] = [];
  for (const item of cases) {
    const response = await fetch(`${session.endpoint}/capture`, {
      method: "POST",
      headers: {
        Origin: item.origin,
        Authorization: `Bearer ${item.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ artPath, update: true }),
    });
    assert.equal(response.status, item.status, item.name);
    receipts.push({ name: item.name, status: response.status });
    assert.deepEqual(await storedCaptureBytes(directory), initial);
  }
  for (const [name, input, status] of [
    ["unknown-art", { artPath: "unknown", update: true }, 404],
    ["malformed-input", { artPath, update: "yes" }, 400],
    ["extra-input", { artPath, update: false, snapshotDir: "/tmp/foreign" }, 400],
  ] as const) {
    const response = await fetch(`${session.endpoint}/capture`, {
      method: "POST",
      headers: {
        Origin: origin,
        Authorization: `Bearer ${session.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(input),
    });
    assert.equal(response.status, status, name);
    assert.deepEqual(await storedCaptureBytes(directory), initial);
    receipts.push({ name, status: response.status });
  }
  return receipts;
}

export async function refuseHostedNavigation(
  session: { endpoint: string; token: string },
  host: { origin: string; redirects: Map<string, string> },
  directory: string,
  artifacts: string,
  artPath: string,
) {
  const baseline = await storedCaptureBytes(artifacts);
  const manifestFile = path.join(directory, "gallery/api/static.json");
  const original = await readFile(manifestFile);
  const manifest = JSON.parse(original.toString()) as {
    previews: Record<string, Record<string, string>>;
  };
  const preview = new URL(manifest.previews[artPath].Default, `${host.origin}/built/gallery/`);
  const receipts: { name: string; status: number; error: string }[] = [];
  async function refuse(name: string) {
    const response = await fetch(`${session.endpoint}/capture`, {
      method: "POST",
      headers: {
        Origin: host.origin,
        Authorization: `Bearer ${session.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ artPath, update: true }),
    });
    assert.equal(response.status, 400, name);
    const data = (await response.json()) as { error: string; success?: boolean };
    assert.notEqual(data.success, true);
    assert.ok(data.error);
    assert.deepEqual(await storedCaptureBytes(artifacts), baseline, name);
    receipts.push({ name, status: response.status, error: data.error });
  }
  try {
    host.redirects.set("/built/gallery/api/static.json", "https://foreign.example/manifest.json");
    await refuse("manifest-redirect");
    host.redirects.clear();
    host.redirects.set(preview.pathname, "https://foreign.example/preview.html");
    await refuse("preview-redirect");
    host.redirects.clear();
    for (const target of [`${host.origin}/outside-gallery`, "https://foreign.example/outside"]) {
      preview.searchParams.set("navigation", target);
      manifest.previews[artPath].Default = preview.href;
      await writeFile(manifestFile, JSON.stringify(manifest));
      await refuse(target.startsWith(host.origin) ? "outside-prefix-document" : "foreign-document");
    }
  } finally {
    host.redirects.clear();
    await writeFile(manifestFile, original);
  }
  return receipts;
}

export async function proveSerialCapture(
  session: { endpoint: string; token: string },
  origin: string,
  artPath: string,
) {
  const responses = await Promise.all(
    [1, 2].map(() =>
      fetch(`${session.endpoint}/capture`, {
        method: "POST",
        headers: {
          Origin: origin,
          Authorization: `Bearer ${session.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ artPath, update: false }),
      }),
    ),
  );
  assert.deepEqual(
    responses.map((item) => item.status).sort((a, b) => a - b),
    [200, 409],
  );
  const complete = (await responses.find((item) => item.status === 200)!.json()) as {
    success: boolean;
    summary: { failed: number; errors: number };
  };
  assert.equal(complete.success, true);
  assert.equal(complete.summary.failed, 0);
  assert.equal(complete.summary.errors, 0);
  return { statuses: [200, 409], summary: complete.summary };
}
