import assert from "node:assert/strict";
import { patchCssModuleGenerateScopedName } from "./css-modules.ts";

{
  const filenames: string[] = [];
  const config = {
    css: {
      modules: {
        generateScopedName(name: string, filename: string) {
          filenames.push(filename);
          return `${filename}:${name}`;
        },
      },
    },
  };

  patchCssModuleGenerateScopedName(config);

  assert.equal(
    config.css.modules.generateScopedName(
      "root",
      "\0/@fs/Users/example/app/src/Button.vue?vue=&type=style&index=0&lang=scss&module=.module.scss",
      "",
    ),
    "/Users/example/app/src/Button.vue:root",
    "CSS module virtual IDs should pass the real file path to generateScopedName",
  );
  assert.deepEqual(filenames, ["/Users/example/app/src/Button.vue"]);
}

{
  const config = {
    css: {
      modules: {
        generateScopedName(name: string, filename: string) {
          return `${filename}:${name}`;
        },
      },
    },
  };

  patchCssModuleGenerateScopedName(config);

  assert.equal(
    config.css.modules.generateScopedName(
      "root",
      "/Users/example/app/src/Button.vue?vue&type=style&index=0&lang=css&module",
      "",
    ),
    "/Users/example/app/src/Button.vue:root",
    "regular Vue CSS module IDs should still be query-stripped",
  );
}

console.log("✅ vite-plugin-vize css modules tests passed!");
