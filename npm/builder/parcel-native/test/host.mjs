import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const fixture = path.join(packageRoot, "test/fixture");
const output = path.join(fixture, "dist");
const cache = path.join(fixture, ".parcel-cache");
const scopeRoot = path.join(fixture, "node_modules/@vizejs");
const installedPlugin = path.join(scopeRoot, "parcel-transformer-native");
fs.rmSync(output, { recursive: true, force: true });
fs.rmSync(cache, { recursive: true, force: true });
fs.mkdirSync(scopeRoot, { recursive: true });
fs.symlinkSync(packageRoot, installedPlugin, process.platform === "win32" ? "junction" : "dir");

const parcel = path.join(packageRoot, "node_modules/parcel/bin/parcel");
try {
  const result = spawnSync(
    process.execPath,
    [
      parcel,
      "build",
      path.join(fixture, "index.html"),
      "--config",
      path.join(fixture, ".parcelrc"),
      "--dist-dir",
      output,
    ],
    { cwd: fixture, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.signal, null);

  const files = fs.readdirSync(output).sort();
  assert(
    files.some((file) => file.endsWith(".html")),
    "Parcel emitted HTML",
  );
  assert(
    files.some((file) => file.endsWith(".js")),
    "Parcel emitted JavaScript",
  );
  assert(
    files.some((file) => file.endsWith(".svg")),
    "Parcel emitted the template asset",
  );
  const javascript = files
    .filter((file) => file.endsWith(".js"))
    .map((file) => fs.readFileSync(path.join(output, file), "utf8"))
    .join("\n");
  assert.match(javascript, /from-native-parcel/u);
  assert.match(javascript, /vize-style-/u);
  assert.match(javascript, /rebeccapurple/u);
  assert.doesNotMatch(javascript, /<template>/u);
} finally {
  fs.rmSync(output, { recursive: true, force: true });
  fs.rmSync(cache, { recursive: true, force: true });
  fs.unlinkSync(installedPlugin);
  fs.rmSync(path.join(fixture, "node_modules"), { recursive: true, force: true });
}
