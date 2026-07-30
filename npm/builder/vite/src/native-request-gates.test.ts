/**
 * Soundness matrix for the string pre-gates that keep module IDs off the native
 * request classifier (#3427).
 *
 * A production build of 300 SFCs made 6,934 `classifyVitePluginRequest` NAPI
 * calls -- 23 per SFC -- and most of them existed only to be told the id is not
 * Vize's. Each gate below replaces such a call with a string test, so each gate
 * has to be *weaker* than the native fact it stands in for: every id the
 * classifier would have accepted must still pass. These tests assert exactly
 * that, against the real native classifier, over a matrix of realistic and
 * adversarial ids -- decoy `/@fs` prefixes, Windows drive letters, a `.vue`
 * directory component, and the three id prefixes Vite and the plugin use.
 *
 * `fromPluginVisibleVirtualId` additionally dropped a second classify of
 * `request.normalizedFsId ?? id` in favour of a string expression, so its whole
 * 252-id result vector is compared against the pre-#3427 implementation.
 */

import assert from "node:assert/strict";

import { classifyVitePluginRequest } from "@vizejs/native";

import { fromPluginVisibleVirtualId } from "./virtual.ts";
import { isPotentialVizeImporter } from "./plugin/resolve.ts";
import { normalizeVirtualStyleId, transformScopedPreprocessorCss } from "./plugin/compat.ts";

/** Path shapes: every `.vue`-ish and non-`.vue` ending, with `/@fs` decoys. */
const PATHS = [
  "/repo/app/Foo.vue.ts",
  "/repo/app/Foo.vue.tsx",
  "/repo/app/Foo.vue",
  "/repo/app/Foo.ts",
  "/@fs/repo/app/Foo.vue.ts",
  "/@fs/repo/app/Foo.vue.tsx",
  "/@fs/repo/app/Foo.vue",
  "/@fs/repo/app/Foo.ts",
  // `/@fsnot` starts with the literal `/@fs`, so it is stripped by the native
  // `normalized_fs_id` too; the JS expression must strip it identically.
  "/@fsnot/repo/app/Foo.vue.ts",
  "C:/repo/app/Foo.vue.ts",
  "/repo/app.vue/Foo.vue.ts",
  "/@fs/C:/repo/app/Foo.vue.ts",
];

const QUERIES = [
  "",
  "?vue&vize",
  "?vue&vize-ssr",
  "?vue&vize&used=true",
  "?vue&type=style&index=0&lang=scss&scoped=data-v-1a2b3c4d",
  "?macro=true",
  "?vue&vize&t=1700000000000",
];

const PREFIXES = ["", "\0", "\0vize-ssr:"];

const MATRIX = PREFIXES.flatMap((prefix) =>
  PATHS.flatMap((path) => QUERIES.map((query) => `${prefix}${path}${query}`)),
);

assert.equal(MATRIX.length, 252, "the matrix is the 12 x 7 x 3 cross product");

/** `fromPluginVisibleVirtualId` exactly as it read before #3427. */
function fromPluginVisibleVirtualIdBefore(id: string): string | null {
  if (id.startsWith("\0")) {
    return null;
  }
  const request = classifyVitePluginRequest(id);
  const isVirtualPath = request.path.endsWith(".vue.ts") || request.path.endsWith(".vue.tsx");
  if (!isVirtualPath || !request.querySuffix) {
    return null;
  }
  const params = new URLSearchParams(request.querySuffix.slice(1));
  if (!params.has("vue") || (!params.has("vize") && !params.has("vize-ssr"))) {
    return null;
  }
  const normalizedRequest = classifyVitePluginRequest(request.normalizedFsId ?? id);
  const normalizedPath = normalizedRequest.path;
  if (normalizedPath.endsWith(".vue.tsx")) {
    return normalizedPath.slice(0, -4);
  }
  return normalizedPath.endsWith(".vue.ts") ? normalizedPath.slice(0, -3) : normalizedPath;
}

assert.deepEqual(
  MATRIX.map((id) => [id, fromPluginVisibleVirtualId(id)]),
  MATRIX.map((id) => [id, fromPluginVisibleVirtualIdBefore(id)]),
  "dropping the second classify and adding the `.vue.ts`/`?` pre-gate must not change any result",
);

// The pre-gate must not be the only thing standing between an id and a non-null
// result: every id that resolves must also satisfy the gate on its own.
assert.deepEqual(
  MATRIX.filter(
    (id) =>
      fromPluginVisibleVirtualIdBefore(id) !== null &&
      !(!id.startsWith("\0") && id.includes(".vue.ts") && id.includes("?")),
  ),
  [],
  "`.vue.ts` + `?` must be implied by a non-null fromPluginVisibleVirtualId",
);

// `isPotentialVizeImporter` traded `classifyVitePluginRequest(importer).isVueSfcPath`
// for `importer.includes(".vue")`.
assert.deepEqual(
  MATRIX.filter(
    (importer) =>
      classifyVitePluginRequest(importer).isVueSfcPath && !isPotentialVizeImporter(importer),
  ),
  [],
  "`.vue` must be implied by the native isVueSfcPath",
);

assert.equal(
  isPotentialVizeImporter(undefined),
  false,
  "an absent importer is not a Vize importer",
);

/**
 * Style IDs as the post-transform plugin actually sees them: the plugin emits
 * `vue=&type=style`, Vite re-serializes it as `vue&type=style`, and the id
 * reaching `transform` carries the virtual extension suffix and sometimes a `\0`.
 */
const STYLE_IDS = [
  "/repo/app/Foo.vue?vue=&type=style&index=0&scoped=data-v-1a2b3c4d&lang=scss.scss",
  "/repo/app/Foo.vue?vue&type=style&index=0&scoped=data-v-1a2b3c4d&lang=scss.scss",
  "\0/repo/app/Foo.vue?vue&type=style&index=0&scoped=data-v-1a2b3c4d&lang=less.less",
  "/repo/app/Foo.vue?vue&type=style&index=1&scoped=data-v-1a2b3c4d&lang=scss&module=.module.scss",
  "/repo/app/Foo.vue?vue&type=style&index=0&lang=scss.scss",
  "/repo/app/Foo.vue?vue&type=style&index=0&scoped=data-v-1a2b3c4d&lang=css.css",
  "/repo/app/Foo.vue?other=1&vue&type=style&index=0&scoped=data-v-1a2b3c4d&lang=styl.styl",
];

// `transformScopedPreprocessorCss` traded a classify of every module in the
// graph for `id.includes("type=style")`.
assert.deepEqual(
  [...MATRIX, ...STYLE_IDS].filter(
    (id) =>
      classifyVitePluginRequest(normalizeVirtualStyleId(id)).isVueStyleQuery &&
      !id.includes("type=style"),
  ),
  [],
  "`type=style` must be implied by the native isVueStyleQuery",
);

// The gate must not have changed what the hook actually rewrites. The last two
// are the hook's own rejections -- no `scoped`, and plain `css` which Vite's
// pipeline already scopes -- and must stay rejections rather than becoming
// gate misses.
const SCOPED = ".a[data-v-1a2b3c4d]{color: red}";
assert.deepEqual(
  STYLE_IDS.map((id) => [id, transformScopedPreprocessorCss(".a { color: red }", id)]),
  [
    [STYLE_IDS[0], SCOPED],
    [STYLE_IDS[1], SCOPED],
    [STYLE_IDS[2], SCOPED],
    [STYLE_IDS[3], SCOPED],
    [STYLE_IDS[4], null],
    [STYLE_IDS[5], null],
    [STYLE_IDS[6], SCOPED],
  ],
  "scoped preprocessor CSS rewriting is unchanged by the pre-gate",
);

console.log("✅ vite-plugin-vize native request gate tests passed!");
