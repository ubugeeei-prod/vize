import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { resolveNuxtCompilerOptions } from "./options.ts";

type VueCopies = { app: string; nuxt: string };

function withVueCopies(versions: VueCopies, run: (appDir: string) => void): void {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-vapor-")));
  const appDir = path.join(root, "app");
  try {
    writeJson(path.join(appDir, "package.json"), {
      name: "app",
      dependencies: { nuxt: "4.5.2", vue: versions.app },
    });
    writeJson(path.join(appDir, "node_modules", "vue", "package.json"), {
      name: "vue",
      version: versions.app,
    });
    writeJson(path.join(appDir, "node_modules", "nuxt", "package.json"), {
      name: "nuxt",
      version: "4.5.2",
    });
    writeJson(path.join(appDir, "node_modules", "nuxt", "node_modules", "vue", "package.json"), {
      name: "vue",
      version: versions.nuxt,
    });
    run(appDir);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function writeJson(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value));
}

function nestedVue(appDir: string): string {
  return fs.realpathSync(path.join(appDir, "node_modules", "nuxt", "node_modules", "vue"));
}

function appVue(appDir: string): string {
  return fs.realpathSync(path.join(appDir, "node_modules", "vue"));
}

function thrownMessage(run: () => void): string {
  let message = "";
  assert.throws(run, (error: unknown) => {
    assert.ok(error instanceof Error);
    assert.equal(error.name, "Error");
    message = error.message;
    return true;
  });
  return message;
}

void test("vapor throws for nested nuxt vue 3.5.44 even when the app vue is 3.6.0-rc.12", () => {
  withVueCopies({ app: "3.6.0-rc.12", nuxt: "3.5.44" }, (appDir) => {
    const message = thrownMessage(() =>
      resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", { vapor: true }),
    );
    assert.match(message, /3\.5\.44/);
    assert.ok(message.includes(nestedVue(appDir)));
    assert.match(message, /nuxt\/node_modules\/vue/);
    assert.equal(message.includes("3.6.0-rc.12"), false);
    assert.equal(message.includes(appVue(appDir)), false);
  });
});

void test("vapor does not throw when the app and nested nuxt vue copies are 3.6.0-rc.12", () => {
  withVueCopies({ app: "3.6.0-rc.12", nuxt: "3.6.0-rc.12" }, (appDir) => {
    const resolved = resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", { vapor: true });
    assert.notEqual(resolved, false);
    assert.equal(resolved !== false && resolved.vapor, true);
    assert.equal(resolved !== false && resolved.handleNodeModulesVue, true);
  });
});

void test("vapor accepts Vue 3.6.0 and 3.7.0", () => {
  for (const version of ["3.6.0", "3.7.0"]) {
    withVueCopies({ app: version, nuxt: version }, (appDir) => {
      const resolved = resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", { vapor: true });
      assert.equal(resolved !== false && resolved.vapor, true, version);
    });
  }
});

void test("vapor off does not throw when nested nuxt vue is 3.5.44", () => {
  withVueCopies({ app: "3.5.44", nuxt: "3.5.44" }, (appDir) => {
    const resolved = resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", { vapor: false });
    assert.notEqual(resolved, false);
    assert.equal(resolved !== false && resolved.vapor, false);
    const enabled = resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", true);
    assert.notEqual(enabled, false);
    assert.equal(enabled !== false && enabled.vapor, undefined);
  });
});

void test("handleNodeModulesVue false throws for the app vue copy only", () => {
  withVueCopies({ app: "3.5.44", nuxt: "3.5.44" }, (appDir) => {
    const message = thrownMessage(() =>
      resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", {
        vapor: true,
        handleNodeModulesVue: false,
      }),
    );
    assert.match(message, /3\.5\.44/);
    assert.ok(message.includes(appVue(appDir)));
    assert.equal(message.includes(nestedVue(appDir)), false);
    assert.equal(message.includes("nuxt/node_modules/vue"), false);
  });
});

void test("handleNodeModulesVue false ignores nested nuxt vue 3.5.44 when the app vue is 3.6", () => {
  withVueCopies({ app: "3.6.0-rc.12", nuxt: "3.5.44" }, (appDir) => {
    const resolved = resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", {
      vapor: true,
      handleNodeModulesVue: false,
    });
    assert.notEqual(resolved, false);
    assert.equal(resolved !== false && resolved.vapor, true);
    assert.equal(resolved !== false && resolved.handleNodeModulesVue, false);
  });
});

void test("one vapor diagnostic names every Vue copy older than 3.6", () => {
  withVueCopies({ app: "3.5.44", nuxt: "3.5.44" }, (appDir) => {
    const message = thrownMessage(() =>
      resolveNuxtCompilerOptions(appDir, "/", "/_nuxt/", { vapor: true }),
    );
    assert.equal(message.includes(appVue(appDir)), true);
    assert.equal(message.includes(nestedVue(appDir)), true);
    assert.equal(message.split("vue@3.5.44").length - 1, 2);
  });
});
