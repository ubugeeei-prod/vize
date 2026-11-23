import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
const root = "npm/native";
const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const nativeImages = readdirSync(root).filter(
  (file) => file.startsWith("vize-vitrine.") && file.endsWith(".node"),
);
assert.equal(nativeImages.length, 1);
const output = join(requiredEnv("VIZE_ESLINT_EVIDENCE"), "native-package");
mkdirSync(output, { recursive: true });
for (const file of ["package.json", ...manifest.files, ...nativeImages]) {
  copyFileSync(join(root, file), join(output, file));
}
