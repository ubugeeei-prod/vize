import { readFileSync } from "node:fs";

interface PackageManifest {
  version: string;
}

const manifest = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
) as PackageManifest;

/** The checked-out or installed create-fresco package version. */
export const CREATE_FRESCO_VERSION = manifest.version;
