import assert from "node:assert/strict";
import { lstatSync, readdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { readJsonRecord } from "../../json-value.ts";
import { fileImage, inside, type FileImage } from "../computed_event_custody.ts";

/** Whole physical TypeScript and Vue package trees, not just version manifests. */
export function libraryImages(modules: string) {
  const names = [
    "typescript",
    "typescript-native",
    "vue",
    ...readdirSync(join(modules, "@vue"))
      .sort()
      .map((name) => "@vue/" + name),
  ];
  const images: FileImage[] = [],
    seen = new Set<string>();
  function visit(filename: string) {
    const actual = realpathSync(filename);
    assert.ok(inside(modules, actual));
    if (seen.has(actual)) return;
    seen.add(actual);
    const stat = lstatSync(actual);
    if (stat.isDirectory())
      for (const name of readdirSync(actual).sort()) visit(join(actual, name));
    else {
      assert.ok(stat.isFile());
      images.push(fileImage(actual));
    }
  }
  for (const name of names) {
    const root = join(modules, name),
      manifest = readJsonRecord(join(root, "package.json"));
    assert.equal(manifest.name, name === "typescript-native" ? "typescript" : name);
    assert.equal(manifest.version, libraryVersion(name));
    visit(root);
  }
  assert.ok(images.length > names.length);
  return { names, files: images };
}
export function libraryVersion(name: string) {
  return name === "typescript" ? "6.0.3" : name === "typescript-native" ? "7.1.0-beta" : "3.5.42";
}
