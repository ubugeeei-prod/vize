import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

/** Bind built source products without installing historical compiler optionals. */
export function prepareNuxt2BridgeSource(
  root: string,
  fixture: string,
  artifacts: string,
): () => void {
  const sources: Record<string, string> = {
    "@vizejs/nuxt": "npm/framework/nuxt",
    "@vizejs/nuxt-lint-config": "npm/framework/nuxt-lint-config",
    "@vizejs/native": "npm/native",
    "oxlint-plugin-vize": "npm/oxlint",
    vize: "npm/cli",
  };
  const installed: string[] = [];
  const identities: unknown[] = [];
  const cleanup = () => {
    for (const target of [...installed].reverse())
      fs.rmSync(target, { recursive: true, force: true });
  };
  try {
    for (const [name, relative] of Object.entries(sources)) {
      const source = path.join(root, relative);
      const target = path.join(fixture, "node_modules", name);
      assert.equal(fs.existsSync(target), false, `${name} is an explicit source product`);
      installed.push(target);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      if (name === "@vizejs/nuxt" || name === "@vizejs/nuxt-lint-config") {
        fs.mkdirSync(target);
        fs.copyFileSync(path.join(source, "package.json"), path.join(target, "package.json"));
        fs.cpSync(path.join(source, "dist"), path.join(target, "dist"), { recursive: true });
      } else {
        fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
      }
      const manifest = fs.readFileSync(path.join(target, "package.json"));
      identities.push({
        name,
        source,
        target,
        manifestSha256: createHash("sha256").update(manifest).digest("hex"),
        manifest: JSON.parse(manifest.toString()),
      });
    }
    // The module retains its Nuxt 2-safe Kit pin; the Bridge host independently
    // resolves its own newer Kit. Vite deliberately resolves from the consumer.
    const kit = path.join(fixture, "node_modules/@vizejs/nuxt/node_modules/@nuxt/kit");
    fs.mkdirSync(path.dirname(kit), { recursive: true });
    fs.symlinkSync(
      path.join(root, "npm/framework/nuxt/node_modules/@nuxt/kit"),
      kit,
      process.platform === "win32" ? "junction" : "dir",
    );
    const hostRequire = createRequire(path.join(fixture, "node_modules/@nuxt/bridge/package.json"));
    const kits = [
      { role: "vize-module", file: path.join(kit, "package.json") },
      { role: "bridge-host", file: hostRequire.resolve("@nuxt/kit/package.json") },
    ].map(({ role, file }) => {
      const bytes = fs.readFileSync(file);
      return {
        role,
        file,
        sha256: createHash("sha256").update(bytes).digest("hex"),
        version: JSON.parse(bytes.toString()).version,
      };
    });
    fs.writeFileSync(
      path.join(artifacts, "bridge-source-products.json"),
      JSON.stringify({ identities, kits }, null, 2) + "\n",
    );
    return cleanup;
  } catch (error) {
    cleanup();
    throw error;
  }
}
