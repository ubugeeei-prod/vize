import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type ConfigurationProperty = {
  type?: string | string[];
  default?: unknown;
  description?: string;
  enum?: string[];
  deprecationMessage?: string;
};

type Manifest = {
  main?: string;
  categories?: string[];
  publisher?: string;
  activationEvents?: string[];
  engines?: { vscode?: string };
  capabilities?: {
    untrustedWorkspaces?: { supported?: string; description?: string };
  };
  contributes?: {
    commands?: Array<{ command?: string; title?: string; category?: string }>;
    menus?: { commandPalette?: Array<{ command?: string; when?: string }> };
    configuration?: { title?: string; properties?: Record<string, ConfigurationProperty> };
    configurationDefaults?: Record<string, unknown>;
    colors?: Array<{ id?: string; defaults?: Record<string, string> }>;
    semanticTokenTypes?: Array<{ id?: string; superType?: string; description?: string }>;
    semanticTokenModifiers?: Array<{ id?: string; description?: string }>;
  };
};

function readManifest(): Manifest {
  return JSON.parse(
    fs.readFileSync(path.join(root, "npm/vscode-vize/package.json"), "utf-8"),
  ) as Manifest;
}

const LANGUAGE_SCOPED_COMMANDS = new Set(["vize.restartServer", "vize.showOutput"]);

test("every contributed command has a matching onCommand activation event", () => {
  const manifest = readManifest();
  const commands = (manifest.contributes?.commands ?? [])
    .map((command) => command.command ?? "")
    .filter(Boolean)
    .sort();
  const onCommandEvents = (manifest.activationEvents ?? [])
    .filter((event) => event.startsWith("onCommand:"))
    .map((event) => event.slice("onCommand:".length))
    .sort();

  // A command without an activation event silently fails to start the extension
  // when invoked from a keybinding; an orphan activation event is dead config.
  assert.deepEqual(onCommandEvents, commands);
});

test("command palette menu only references declared commands", () => {
  const manifest = readManifest();
  const declared = new Set(
    (manifest.contributes?.commands ?? []).map((command) => command.command),
  );
  const palette = manifest.contributes?.menus?.commandPalette ?? [];
  assert.ok(palette.length > 0);

  for (const item of palette) {
    assert.ok(
      declared.has(item.command),
      `palette command ${item.command} is not declared in contributes.commands`,
    );
  }
});

test("language-scoped commands are gated by an editor language guard", () => {
  const manifest = readManifest();
  const palette = manifest.contributes?.menus?.commandPalette ?? [];

  for (const item of palette) {
    if (LANGUAGE_SCOPED_COMMANDS.has(item.command ?? "")) {
      assert.match(item.when ?? "", /editorLangId == vue/);
      assert.match(item.when ?? "", /editorLangId == art-vue/);
    } else {
      // Profile/status commands are always available, so they must not carry a
      // language guard that would hide them from the palette.
      assert.equal(item.when, undefined, `${item.command} should not be language-gated`);
    }
  }
});

test("every contributed command carries the Vize category and a title", () => {
  const manifest = readManifest();
  const commands = manifest.contributes?.commands ?? [];
  assert.ok(commands.length > 0);

  for (const command of commands) {
    assert.ok(command.command?.startsWith("vize."), JSON.stringify(command));
    assert.equal(command.category, "Vize", JSON.stringify(command));
    assert.ok(
      typeof command.title === "string" && command.title.length > 0,
      JSON.stringify(command),
    );
  }
});

test("activation events register the supported languages", () => {
  const manifest = readManifest();
  const events = new Set(manifest.activationEvents ?? []);
  for (const language of ["vue", "art-vue", "html"]) {
    assert.ok(events.has(`onLanguage:${language}`), `missing onLanguage:${language}`);
  }
});

test("configuration properties are well-formed and typed", () => {
  const manifest = readManifest();
  const properties = manifest.contributes?.configuration?.properties ?? {};
  const keys = Object.keys(properties);
  assert.ok(keys.length > 0);

  for (const [key, property] of Object.entries(properties)) {
    assert.ok(key.startsWith("vize."), `property ${key} should be namespaced under vize.`);
    assert.ok(property.type !== undefined, `property ${key} should declare a type`);
    assert.ok(
      typeof property.description === "string" && property.description.length > 0,
      `property ${key} should have a description`,
    );
  }

  // Every `vize.<feature>.enable` toggle is a boolean with a boolean default.
  for (const [key, property] of Object.entries(properties)) {
    if (key.endsWith(".enable")) {
      assert.equal(property.type, "boolean", `${key} should be a boolean toggle`);
      assert.equal(typeof property.default, "boolean", `${key} should default to a boolean`);
    }
  }
});

test("trace and server-path configuration keep their published contract", () => {
  const manifest = readManifest();
  const properties = manifest.contributes?.configuration?.properties ?? {};

  const trace = properties["vize.trace.server"];
  assert.ok(trace);
  assert.deepEqual(trace.enum, ["off", "messages", "verbose"]);
  assert.equal(trace.default, "off");

  const serverPath = properties["vize.serverPath"];
  assert.ok(serverPath);
  assert.equal(serverPath.type, "string");
  assert.equal(serverPath.default, "");

  // The deprecated alias must keep pointing developers at the replacement.
  const deprecated = properties["vize.diagnostics.enable"];
  assert.ok(deprecated);
  assert.ok(
    typeof deprecated.deprecationMessage === "string" &&
      /vize\.lint\.enable/.test(deprecated.deprecationMessage),
    "vize.diagnostics.enable should be deprecated in favor of vize.lint.enable",
  );
});

test("semantic token contributions and inlay-hint colors stay registered", () => {
  const manifest = readManifest();

  const tokenTypes = new Map(
    (manifest.contributes?.semanticTokenTypes ?? []).map((entry) => [entry.id, entry]),
  );
  assert.equal(tokenTypes.get("vueDirective")?.superType, "keyword");
  assert.equal(tokenTypes.get("vueComponent")?.superType, "class");

  const tokenModifiers = new Set(
    (manifest.contributes?.semanticTokenModifiers ?? []).map((entry) => entry.id),
  );
  assert.ok(tokenModifiers.has("vue"));

  const colorIds = new Set((manifest.contributes?.colors ?? []).map((color) => color.id));
  assert.ok(colorIds.has("vize.inlayHint.propsForeground"));
  assert.ok(colorIds.has("vize.inlayHint.propsBackground"));

  // Each declared color must provide light/dark defaults so themes never render
  // an undefined inlay-hint color.
  for (const color of manifest.contributes?.colors ?? []) {
    assert.ok(color.defaults?.dark, `${color.id} should declare a dark default`);
    assert.ok(color.defaults?.light, `${color.id} should declare a light default`);
  }
});

test("extension entry point, engine, and trust contract are present", () => {
  const manifest = readManifest();

  // `main` points at a build artifact (dist/extension.cjs) that only exists
  // after `vp pack`, so assert the declared contract rather than the file on
  // disk; the editor-extensions CI job verifies the build output itself.
  assert.equal(manifest.main, "./dist/extension.cjs");

  assert.ok(manifest.engines?.vscode, "manifest should pin a vscode engine range");

  // Vize launches a native binary, so it must declare limited untrusted-workspace
  // support and explain why.
  assert.equal(manifest.capabilities?.untrustedWorkspaces?.supported, "limited");
  assert.ok(
    typeof manifest.capabilities?.untrustedWorkspaces?.description === "string" &&
      manifest.capabilities.untrustedWorkspaces.description.length > 0,
  );

  assert.ok((manifest.categories ?? []).includes("Programming Languages"));
});
