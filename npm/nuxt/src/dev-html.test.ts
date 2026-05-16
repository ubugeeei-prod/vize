import assert from "node:assert/strict";
import { sanitizeNuxtDevStylesheetLinks } from "./dev-html.ts";

const html = `<!DOCTYPE html><html><head>
<link rel="stylesheet" href="/_nuxt/assets/main.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/@fs/Users/me/project/node_modules/vue-data-ui/dist/style.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/__uno.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/Users/me/project/app/assets/main.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/vue-data-ui/style.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/%00/Users/me/project/app/pages/index.vue?vue=&type=style&index=0&lang=css&module=.module.css" crossorigin>
</head></html>`;

assert.equal(
  sanitizeNuxtDevStylesheetLinks(html),
  `<!DOCTYPE html><html><head>
<link rel="stylesheet" href="/_nuxt/assets/main.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/@fs/Users/me/project/node_modules/vue-data-ui/dist/style.css" crossorigin>
<link rel="stylesheet" href="/_nuxt/__uno.css" crossorigin>



</head></html>`,
  "should strip broken dev-only stylesheet links while keeping valid assets",
);

assert.equal(
  sanitizeNuxtDevStylesheetLinks(
    `<link rel="stylesheet" href="/_nuxt/%40fs/Users/me/project/node_modules/pkg/style.css"><link rel="stylesheet" href="/_nuxt/assets%2Fentry.css">`,
  ),
  `<link rel="stylesheet" href="/_nuxt/%40fs/Users/me/project/node_modules/pkg/style.css"><link rel="stylesheet" href="/_nuxt/assets%2Fentry.css">`,
  "URL-encoded valid Nuxt dev asset paths should be preserved",
);

assert.equal(
  sanitizeNuxtDevStylesheetLinks(
    `<link rel="stylesheet" href="/_nuxt/assets/%2e%2e/server.css"><link rel="stylesheet" href="/_nuxt/@fs/C:%5Cproject%5C..%5Csecret.css"><link rel="stylesheet" href="/_nuxt/assets/%00secret.css">`,
  ),
  "",
  "decoded traversal and null-byte paths should be stripped",
);

assert.equal(
  sanitizeNuxtDevStylesheetLinks(
    `<link rel="stylesheet" href="/docs/_nuxt/assets/main.css"><link rel="stylesheet" href="/docs/_nuxt/pkg/style.css">`,
    "/docs/_nuxt/",
  ),
  `<link rel="stylesheet" href="/docs/_nuxt/assets/main.css">`,
  "custom buildAssetsDir should be honored when filtering stylesheet links",
);

assert.equal(
  sanitizeNuxtDevStylesheetLinks(
    `<link rel="stylesheet" href="/_nuxt/assets/main.css"><link rel="stylesheet" href="/_nuxt/assets/main.css">`,
  ),
  `<link rel="stylesheet" href="/_nuxt/assets/main.css">`,
  "duplicate stylesheet hrefs should be removed",
);

console.log("✅ nuxt dev html tests passed!");
