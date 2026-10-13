import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fixtureRoot } from "./custody.mjs";
export const projectInputs = ["nuxt.config.ts", "app/app.vue", "app/pages/index.vue"];
export function projectDependencies(version) {
  assert.match(version, /^0\.[1-9][0-9]*\.0$/);
  return {
    nuxt: "4.5.0",
    vite: "8.3.4",
    vue: "3.5.40",
    typescript: "7.0.2",
    playwright: "1.62.1",
    ...Object.fromEntries(
      [
        "@vizejs/nuxt",
        "@vizejs/vite-plugin",
        "@vizejs/unplugin",
        "@vizejs/native",
        "@vizejs/native-linux-x64-gnu",
      ].map((name) => [name, version]),
    ),
  };
}
export function copyProject(destination, fixture, dependencies, variant, version = "0.443.0") {
  assert.equal(fs.existsSync(destination), false, "fresh project required");
  for (const name of projectInputs) {
    const target = path.join(destination, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    const original = fixture.source.get(`${fixtureRoot}/${name}`);
    assert(original);
    const bytes =
      variant === "stock" && name === "nuxt.config.ts"
        ? Buffer.from(original.toString("utf8").replace('modules: ["@vizejs/nuxt"]', "modules: []"))
        : original;
    if (variant === "stock" && name === "nuxt.config.ts")
      assert.equal(
        bytes.toString("utf8").replace("modules: []", 'modules: ["@vizejs/nuxt"]'),
        original.toString("utf8"),
      );
    fs.writeFileSync(target, bytes, { flag: "wx" });
  }
  const originalManifest = fixture.source.get(`${fixtureRoot}/package.json`);
  const manifest = JSON.parse(originalManifest);
  assert.equal(manifest.dependencies["@vizejs/nuxt"], "0.443.0");
  const bytes =
    variant === "published"
      ? Buffer.from(
          originalManifest
            .toString("utf8")
            .replace('"@vizejs/nuxt": "0.443.0"', `"@vizejs/nuxt": "${version}"`),
        )
      : originalManifest;
  if (variant === "published")
    assert.equal(
      bytes.toString("utf8").replace(`"@vizejs/nuxt": "${version}"`, '"@vizejs/nuxt": "0.443.0"'),
      originalManifest.toString("utf8"),
    );
  fs.writeFileSync(path.join(destination, "package.json"), bytes, { flag: "wx" });
  // Link only this campaign's authenticated public installation; never source
  // adapters or checkout modules. Original SFC/config bytes remain untouched.
  fs.symlinkSync(
    path.join(dependencies, "node_modules"),
    path.join(destination, "node_modules"),
    "dir",
  );
  return destination;
}
