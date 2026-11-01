// An independently labeled image probe. Its PID is never a compiler/CLI PID.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync, realpathSync } from "node:fs";
import { fileImage, inside } from "../computed_event_custody.ts";
import { object, text } from "./value.ts";
export function probe(root: string): unknown {
  const require = createRequire(join(root, "node_modules/vize/package.json"));
  const native = object(require("@vizejs/native"));
  const images = Object.keys(require.cache).filter((p) => p.endsWith(".node") && inside(root, p));
  assert.equal(images.length, 1);
  const image = fileImage(images[0]!);
  const exports = object(require.cache[image.path]?.exports);
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof exports.getNativeBuildToken, "function");
  const token = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []);
  assert.equal(token, Reflect.apply(exports.getNativeBuildToken as () => unknown, exports, []));
  assert.match(text(token), /^[a-f0-9]{32}$/u);
  const entry = realpathSync(require.resolve("@vizejs/native"));
  const helper = object(require(join(dirname(entry), "native-binding-identity.js")));
  const identity = Reflect.apply(helper.getBindingIdentity as (v: unknown) => unknown, helper, [
    exports,
  ]);
  assert.deepEqual(identity, { kind: "native", token });
  const pkg = object(JSON.parse(readFileSync(join(dirname(image.path), "package.json"), "utf8")));
  return {
    ...image,
    packageName: text(pkg.name),
    token,
    identityProbeOnly: true,
    probePid: process.pid,
  };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (!process.argv[2]) throw new Error("installed consumer root required");
  process.stdout.write(JSON.stringify(probe(realpathSync(process.argv[2]))) + "\n");
}
