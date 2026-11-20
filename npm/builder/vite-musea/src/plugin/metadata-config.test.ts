import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import type { ArtFileInfo } from "../types/index.js";
import { externalStatusForArt, resolveExternalStatuses } from "./metadata-config.js";

function createArt(artPath: string, title = "Button"): ArtFileInfo {
  return {
    path: artPath,
    metadata: { title, tags: [] },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

void test("inline statuses prefer an Art path over its title", async () => {
  const root = path.resolve("project");
  const art = createArt(path.join(root, "src", "Button.art.vue"));
  const resolved = await resolveExternalStatuses(root, {
    Button: { status: "success", label: "In sync" },
    "src/Button.art.vue": "warning",
  });

  assert.deepEqual(externalStatusForArt(art, root, resolved.values), { status: "warning" });
});

void test("JSON status files accept a nested statuses object and portable separators", async () => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-statuses-"));
  const art = createArt(path.join(root, "src", "Button.art.vue"));

  try {
    await fs.promises.writeFile(
      path.join(root, "statuses.json"),
      JSON.stringify({
        statuses: {
          "src\\Button.art.vue": { status: "error", label: "Figma changed" },
        },
      }),
    );

    const resolved = await resolveExternalStatuses(root, "statuses.json");

    assert.equal(resolved.filePath, path.join(root, "statuses.json"));
    assert.deepEqual(externalStatusForArt(art, root, resolved.values), {
      status: "error",
      label: "Figma changed",
    });
  } finally {
    await fs.promises.rm(root, { recursive: true, force: true });
  }
});

void test("external statuses reject invalid values and paths outside the project", async () => {
  const root = path.resolve("project");

  await assert.rejects(
    resolveExternalStatuses(root, { Button: "draft" as "success" }),
    /invalid external status/u,
  );
  await assert.rejects(resolveExternalStatuses(root, "../statuses.json"), /escapes/u);
});
