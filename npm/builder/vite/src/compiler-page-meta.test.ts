/** #7821: the public Vite adapter preserves ordinary page metadata. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { compileBatch, compileFile } from "./compiler.ts";

for (const name of ["Page", "global", "mixed", "local", "alias", "shadowed"]) {
  const source = fs.readFileSync(
    new URL(
      `../../../../tests/_fixtures/differential/compiler/page-meta-runtime/${name}.vue.txt`,
      import.meta.url,
    ),
    "utf8",
  );
  const file = { path: `/src/${name}.vue`, source };
  for (const ssr of [false, true]) {
    for (const vapor of [false, true]) {
      for (const nuxtPageMeta of [false, true]) {
        const options = { sourceMap: true, ssr, vapor, nuxtPageMeta };
        const single = compileFile(file.path, new Map(), options, source);
        const batch = compileBatch([file], new Map(), options);
        const result = batch.results[0];
        assert.deepEqual(result.errors, []);
        assert.equal(result.code, single.code, "single and batch use the same ownership");
        const extracted = nuxtPageMeta && ["Page", "global", "mixed"].includes(name);
        assert.equal(single.macroArtifacts?.length, Number(extracted));
        assert.equal(result.macroArtifacts?.length, Number(extracted));
        assert.equal(
          single.code.includes(name === "alias" ? "pageMeta(" : "definePageMeta("),
          !extracted,
        );
        if (name === "mixed") assert.match(single.code, /useRoute/);
        if (name === "Page") assert.equal(single.code.includes("#imports"), !extracted);
      }
    }
  }
}
