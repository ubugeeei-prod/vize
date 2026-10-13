import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const digest = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex");
export const fixtureRoots = [
  "examples/vite-musea/src/",
  "tests/tooling/fixtures/musea/static-variant-name/",
  "tests/tooling/fixtures/musea/variant-binding-collision/",
];
export const drivers = [
  ["tools/support/release/public_acceptance/gallery.ts", "gallery.ts"],
  ["npm/builder/vite-musea/src/inline-art-static.fixtures.ts", "inline-art-static.fixtures.ts"],
  [
    "npm/builder/vite-musea/src/inline-art-static.browser.test.ts",
    "inline-art-static.browser.test.ts",
  ],
  [
    "npm/builder/vite-musea/src/variant-bindings-native.fixtures.ts",
    "variant-bindings-native.fixtures.ts",
  ],
  [
    "npm/builder/vite-musea/src/variant-bindings-native.browser.test.ts",
    "variant-bindings-native.browser.test.ts",
  ],
] as const;

/** Explicit provider/path/type adapters and serialized DOM fields retain every original assertion. */
export function adaptDriver(file: string, source: string, consumer: string) {
  const replacements: { from: string; to: string }[] = [];
  let bytes = source;
  const replace = (from: string, to: string) => {
    assert.equal(bytes.split(from).length, 2, `${file}: unique adapter source required`);
    bytes = bytes.replace(from, to);
    replacements.push({ from, to });
  };
  const authority = path.join(consumer, "authority");
  const gallery = path.join(
    consumer,
    "node_modules/@vizejs/vite-plugin-musea/dist/gallery/index.html",
  );
  if (file.endsWith(".fixtures.ts")) {
    replace(
      'import vize from "../../vite/src/index.ts";',
      'import vize from "@vizejs/vite-plugin";',
    );
    replace(
      'import { musea } from "./plugin/index.ts";',
      'import { musea } from "@vizejs/vite-plugin-musea";',
    );
    replace(
      'import type { StaticGalleryPayload } from "./static-data.ts";',
      "type StaticGalleryPayload = { arts: { path: string; metadata: { title: string }; variants: { name: string; isDefault: boolean }[] }[]; previews: Record<string, Record<string, string>> };",
    );
    if (file === "inline-art-static.fixtures.ts") {
      replace(
        'export const repository = fileURLToPath(new URL("../../../../", import.meta.url));',
        `export const repository = ${JSON.stringify(authority)};`,
      );
      replace(
        'path.join(repository, "npm/builder/vite-musea/dist/gallery/index.html")',
        JSON.stringify(gallery),
      );
    } else
      replace(
        'path.join(repository, "npm/builder/vite-musea/dist/gallery/index.html")',
        JSON.stringify(gallery),
      );
  }
  if (file === "inline-art-static.browser.test.ts")
    replace('"../../../../tools/support/release/public_acceptance/gallery.ts"', '"./gallery.ts"');
  if (file === "gallery.ts")
    replace(
      "url: frame.src,",
      "url: frame.src,\n      html: frame.contentDocument!.documentElement.outerHTML,",
    );
  if (file === "inline-art-static.browser.test.ts")
    replace(
      'setupCalls: Reflect.get(window, "__publicSetupCalls"),',
      'setupCalls: Reflect.get(window, "__publicSetupCalls"),\n            html: document.documentElement.outerHTML,',
    );
  return { bytes, replacements };
}

export function materialize(consumer: string, files: string[], read: (file: string) => Buffer) {
  assert.ok(files.length > 10 && new Set(files).size === files.length);
  assert.ok(files.every((file) => fixtureRoots.some((root) => file.startsWith(root))));
  const records = [];
  const write = (file: string, bytes: Buffer | string) => {
    const destination = path.join(consumer, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes, { flag: "wx" });
    return { file, sha256: digest(bytes) };
  };
  for (const file of [...files, "examples/vite-musea/musea.preview.ts"])
    records.push({ source: file, ...write(`authority/${file}`, read(file)) });
  const adapters = [];
  for (const [source, file] of drivers) {
    const original = read(source);
    records.push({ source, ...write(`originals/${source}`, original) });
    const adapted = adaptDriver(file, original.toString(), consumer);
    adapters.push({
      source,
      ...write(`drivers/${file}`, adapted.bytes),
      replacements: adapted.replacements,
    });
  }
  // Exact original public red and ordinary control identities remain independent assertions.
  for (const [file, expected] of Object.entries({
    "Host.vue": "3087f1c9c435eccf08fedd48050fd124ab5574ac5c85f508b6b561ea5a0e599c",
    "Host.art.vue": "b13eb1f8d2379b0f91a9f2435f6c4305ee53945560d2bcb607ec90a201845dc0",
    "ordinary/Host.art.vue": "29b14c1956a20cf9dd22b74161d97b304bbb4cb9944ed6edc494c6b4a68d85fc",
  }))
    assert.equal(
      digest(read(`tests/tooling/fixtures/musea/static-variant-name/${file}`)),
      expected,
    );
  return { records, adapters };
}
