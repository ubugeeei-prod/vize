#!/usr/bin/env node
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const config = JSON.parse(fs.readFileSync(process.env.VIZE_CSS_RUSTC_RECEIPT_CONFIG, "utf8"));
const [compiler, ...args] = process.argv.slice(2);
const name = args[args.indexOf("--crate-name") + 1];
const measured = ["vize_vitrine", "vize_atelier_sfc", "vize_croquis", "lightningcss"];
if (measured.includes(name) && args.includes("--out-dir")) {
  const out = fs.realpathSync(args[args.indexOf("--out-dir") + 1]);
  assert.ok(out.startsWith(config.target + path.sep));
  assert.equal(process.env.VIZE_NATIVE_BUILD_TOKEN, config.token);
  const source = execFileSync("git", ["-C", config.root, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  assert.equal(source, config.source);
  const file = path.join(config.receipts, `${process.pid}-${name}.json`);
  assert.equal(fs.existsSync(file), false);
  fs.writeFileSync(
    file,
    JSON.stringify(
      {
        source,
        side: config.side,
        crate: name,
        compiler: fs.realpathSync(compiler),
        args,
        cwd: process.cwd(),
        outDir: out,
        target: config.target,
        manifestDir: process.env.CARGO_MANIFEST_DIR,
        token: process.env.VIZE_NATIVE_BUILD_TOKEN,
        wrapper: fileURLToPath(import.meta.url),
      },
      null,
      2,
    ) + "\n",
  );
}
const result = spawnSync(compiler, args, { stdio: "inherit", shell: false });
if (result.error) throw result.error;
assert.equal(result.signal, null);
process.exit(result.status);
