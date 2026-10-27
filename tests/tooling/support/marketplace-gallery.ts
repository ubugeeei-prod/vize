import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "./release-command.ts";

/** The gallery's `ubugeeei.vize` inventory after 0.584.0, trimmed to two releases. */
export function recordedGallery(): {
  results: [
    { extensions: [{ versions: { version: string; targetPlatform?: string; flags?: string }[] }] },
  ];
} {
  const file = path.join(
    repoRoot,
    "tests",
    "_fixtures",
    "release",
    "marketplace-gallery-platforms.json",
  );
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

/** A complete one-extension gallery envelope listing exactly `versions`. */
export function galleryWith(
  versions: { version: string; targetPlatform?: string; flags?: string }[],
) {
  const gallery = recordedGallery();
  gallery.results[0].extensions[0].versions = versions;
  return gallery;
}
