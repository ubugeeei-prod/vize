import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const experimentalKeys = [
  "patternedTemplate",
  "pattenedTemplate",
  "inTagComment",
  "intagComment",
  "selfComponent",
  "strictSlotChildren",
  "serverScript",
  "server script",
  "vapor",
  "jsxVapor",
];

const rfcLinks = [
  "https://github.com/vuejs/rfcs/pull/823",
  "https://github.com/vuejs/rfcs/pull/831",
  "https://github.com/vuejs/rfcs/pull/833",
  "https://github.com/vuejs/rfcs/pull/734",
];

const sectionHeadings = [
  "## Switch Values",
  "## Precedence",
  "## Flag Reference",
  "## Patterned Templates",
  "## In-Tag Comments",
  "## Self Component",
  "## Strict Slot Children",
  "## Server Script",
  "## Vapor",
  "## JSX Vapor",
];

test("experimentals docs fully describe experimental opt-in flags", () => {
  const experimentals = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/experimentals.md"),
    "utf8",
  );

  assert.match(experimentals, /# Experimentals/);
  assert.match(experimentals, /Missing keys, `false`, and `null` are off/);
  assert.match(experimentals, /Direct Vite plugin\s+options win over shared config/);
  assert.match(experimentals, /Do not set a recommended flag and its alias at the same time/);
  assert.match(experimentals, /Typed TypeScript and Pkl config should use/);
  assert.match(
    experimentals,
    /\| `inTagComment` \| Vue RFC \[#831\]\(https:\/\/github\.com\/vuejs\/rfcs\/pull\/831\)/,
  );
  assert.match(
    experimentals,
    /\| `selfComponent` \| Vue RFC \[#833\]\(https:\/\/github\.com\/vuejs\/rfcs\/pull\/833\)/,
  );

  for (const heading of sectionHeadings) {
    assert.match(experimentals, new RegExp(escapeRegExp(heading)), `${heading} must be documented`);
  }
  for (const key of experimentalKeys) {
    assert.match(experimentals, new RegExp(escapeRegExp(key)), `${key} must be documented`);
  }
  for (const link of rfcLinks) {
    assert.match(experimentals, new RegExp(escapeRegExp(link)), `${link} must be linked`);
  }

  assert.match(experimentals, /<template v-match="entry">/);
  assert.match(experimentals, /\/\/ Kept for tooling, stripped from generated output\./);
  assert.match(experimentals, /<Self v-for="child in node\.children"/);
  assert.match(experimentals, /readonly __vizeSlots\?:/);
  assert.match(experimentals, /serverScript: true/);
  assert.match(experimentals, /vapor: true/);
  assert.match(experimentals, /jsxVapor: true/);
});

test("configuration and vite plugin docs link to experimentals", () => {
  const configuration = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/configuration.md"),
    "utf8",
  );
  const vitePlugin = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/vite-plugin.md"),
    "utf8",
  );

  assert.match(configuration, /\[Experimentals\]\(\.\/experimentals\.md\)/);
  assert.match(vitePlugin, /\| `experimentals`/);
  assert.match(vitePlugin, /\[Experimentals\]\(\.\/experimentals\.md\)/);
});

test("cli experimental docs point at the complete experimentals reference", () => {
  const cliReadme = fs.readFileSync(path.join(repoRoot, "npm/cli/README.md"), "utf8");
  const cliDoc = fs.readFileSync(
    path.join(repoRoot, "npm/cli/docs/experimental-vue-rfc-flags.md"),
    "utf8",
  );

  assert.match(cliReadme, /`experimentals\.inTagComment`/);
  assert.match(cliReadme, /historical names `intagComment` and `pattenedTemplate`/);
  assert.doesNotMatch(cliReadme, /`experimentals\.intagComment`/);
  assert.match(cliDoc, /https:\/\/vizejs\.dev\/guide\/experimentals/);
  assert.match(cliDoc, /`serverScript`, `vapor`, and `jsxVapor`/);
  assert.match(cliDoc, /Avoid\s+setting a recommended name and its alias together/);
  assert.doesNotMatch(cliDoc, /`experimentals\.intagComment` enables/);
});

test("vite plugin type comments document experimental switches", () => {
  const types = fs.readFileSync(
    path.join(repoRoot, "npm/builder/vite/src/experimental-options.ts"),
    "utf8",
  );

  assert.match(types, /Missing keys, `false`, and `null` are disabled/);
  assert.match(types, /Direct `vize\(\{ experimentals \}\)` values take precedence/);
  assert.match(types, /Prefer stable `compiler` options/);
  assert.match(types, /Vue RFC #831: parse compile-time-only `\/\/` comments inside start tags/);
  assert.match(types, /Vue RFC #833: reserve `<Self>` for recursive component resolution/);
  assert.match(types, /Vue RFC #734: enable virtual-TypeScript checks/);

  for (const key of experimentalKeys) {
    assert.match(types, new RegExp(escapeRegExp(key)), `${key} must be documented in types`);
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
