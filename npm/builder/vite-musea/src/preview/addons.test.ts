import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { MUSEA_ADDONS_INIT_CODE, MUSEA_PLAY_INIT_CODE } from "./addons.js";
import { MUSEA_PLAY_RUNTIME_CODE } from "./play.js";

void test("the play runtime exposes scoped queries, interactions and assertions", () => {
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /function __museaCreatePlayContext/);
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /getByRole/);
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /userEvent/);
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /step:/);
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /kind: 'assertion'/);
  assert.match(MUSEA_PLAY_RUNTIME_CODE, /toHaveTextContent/);
  assert.match(MUSEA_PLAY_INIT_CODE, /musea:run-play/);
  assert.match(MUSEA_PLAY_INIT_CODE, /musea:play-result/);
  assert.match(MUSEA_PLAY_INIT_CODE, /report: context\.__museaReport/);
});

void test("the a11y runner rejects a vendor response that is not axe-core", () => {
  // axe-core is an optional peer, so a static export can ship without
  // `vendor/axe-core.min.js`. Dev answers that route with a 404, which fires
  // `script.onerror`, but a static host is free to answer a missing asset with
  // an SPA fallback: 200 plus HTML. The script tag then loads successfully and
  // `window.axe` is still undefined, so the next line used to fail with
  // "cannot read properties of undefined (reading 'run')" — an opaque message
  // that points nowhere near the missing dependency.
  const loadBlock = MUSEA_ADDONS_INIT_CODE.slice(
    MUSEA_ADDONS_INIT_CODE.indexOf("musea:run-a11y"),
    MUSEA_ADDONS_INIT_CODE.indexOf("window.axe.run"),
  );

  assert.notEqual(loadBlock, "", "the run-a11y handler must precede the axe.run call");
  assert.match(
    loadBlock,
    /if \(!window\.axe\) \{\s*throw new Error\(/u,
    "the runner must verify window.axe after the vendor script loads, not just await the load event",
  );
  assert.match(
    loadBlock,
    /install axe-core/u,
    "the failure must name the missing dependency so the message is actionable",
  );
  assert.match(
    loadBlock,
    /SPA fallback/u,
    "the failure must mention the served-as-HTML case, which is how a deployed gallery hits this",
  );
});

void test("gallery posts preview commands only to the page origin", () => {
  const galleryPostMessage = fs.readFileSync(
    path.resolve(
      path.dirname(fileURLToPath(import.meta.url)),
      "../../gallery/composables/usePostMessage.ts",
    ),
    "utf8",
  );

  assert.doesNotMatch(
    galleryPostMessage,
    /postMessage\([^)]*,\s*['"]\*['"]\)/,
    "the gallery must not broadcast musea commands to every embedded frame",
  );
  assert.match(
    galleryPostMessage,
    /srcOrigin === pageOrigin \? pageOrigin : null/,
    "commands must stay on the gallery origin so a navigated iframe cannot receive them",
  );
});

void test("preview addon messages stay on the preview origin", () => {
  assert.match(
    MUSEA_ADDONS_INIT_CODE,
    /if \(e\.origin !== window\.location\.origin\) return;/,
    "the iframe must ignore cross-origin postMessage commands",
  );
  assert.match(
    MUSEA_ADDONS_INIT_CODE,
    /postMessage\(\{ type: 'musea:event', payload \}, parentOrigin\)/,
  );
  assert.doesNotMatch(MUSEA_ADDONS_INIT_CODE, /postMessage\([^)]*,\s*['"]\*['"]\)/);
});

void test("preview readiness can be requested after iframe load", () => {
  assert.match(MUSEA_ADDONS_INIT_CODE, /case 'musea:request-ready'/);
  assert.match(
    MUSEA_ADDONS_INIT_CODE,
    /case 'musea:request-ready':[\s\S]*postMessage\(\{ type: 'musea:ready'/u,
  );
});

void test("the a11y runner reports failures back to the gallery instead of throwing away the request", () => {
  // The gallery keys pending requests by requestId and times them out after
  // 30s. A runner that throws without posting a result turns a clear error into
  // a half-minute spinner, so pin that the catch path still answers.
  const catchBlock = MUSEA_ADDONS_INIT_CODE.slice(MUSEA_ADDONS_INIT_CODE.indexOf("window.axe.run"));

  assert.match(
    catchBlock,
    /catch \(err\)[\s\S]*musea:a11y-result[\s\S]*requestId/u,
    "the catch path must post musea:a11y-result carrying the requestId",
  );
});
