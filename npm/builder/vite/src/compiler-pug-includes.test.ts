/**
 * A pug template's `include`s resolve against the SFC and are compile
 * dependencies, so Vite watches them and an edit to an included file
 * routes HMR to the SFC that includes it (#8908).
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { compileFile } from "./compiler.ts";
import { CompiledModuleCache } from "./plugin/compiled-module-cache.ts";

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-includes-"));
const sfcPath = path.join(dir, "Card.vue");
const partial = path.join(dir, "partials", "body.pug");
fs.mkdirSync(path.dirname(partial));
fs.writeFileSync(partial, "p.first First\n");
const source = `<template lang="pug">\n.card\n  include partials/body.pug\n</template>\n`;
const options = { sourceMap: false, ssr: false, vapor: false };

const cache = new CompiledModuleCache();
const first = compileFile(sfcPath, cache, options, source);
assert.deepEqual(first.dependencies, [partial], "the include is a dependency");
assert.match(first.code, /First/);
assert.deepEqual(cache.ownersOf(partial), [sfcPath], "an include edit routes to its SFC");

fs.writeFileSync(partial, "p.second Second\n");
const second = compileFile(sfcPath, cache, options, source);
assert.match(second.code, /Second/, "recompiling reads the edited include");
assert.doesNotMatch(second.code, /First/);

fs.rmSync(dir, { recursive: true, force: true });
