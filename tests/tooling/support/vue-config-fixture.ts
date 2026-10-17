import { writeFileSync } from "node:fs";
import { join } from "node:path";

type ConfigObject = Record<string, unknown>;
type NativeConfigFixture = ConfigObject | string | Uint8Array;

function configObject(value: unknown, label: string): ConfigObject {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${label} must be a native configuration object`);
  }
  return value as ConfigObject;
}

function fixtureDocument(document: NativeConfigFixture): ConfigObject {
  return configObject(
    typeof document === "string"
      ? JSON.parse(document)
      : document instanceof Uint8Array
        ? JSON.parse(new TextDecoder().decode(document))
        : document,
    "Fixture",
  );
}

/**
 * Explicit plugin.config accepts native scoped compiler options that are not
 * part of canonical Vue entries. Carry those fixtures without discovery or
 * canonical rewrites, leaving module options to the real adapter's precedence.
 */
export function inlineVuePluginFixtureOptions(
  moduleOptions: ConfigObject | false | undefined,
  document: NativeConfigFixture | null | undefined,
): ConfigObject | false {
  if (moduleOptions === false) return false;
  const options = structuredClone(moduleOptions ?? {});
  if (document !== null && document !== undefined) {
    options.config = structuredClone(fixtureDocument(document));
  }
  return options;
}

/**
 * Carry existing native JSON test vectors through the canonical Vite API.
 * This is test infrastructure, not support for discovering retired config files.
 * Native fixtures default JSX checking to false; Vite projects default it to true.
 * Document $schema metadata belongs on the Vite carrier, outside Vue options.
 */
export function nativeConfigToVueOptions(document: NativeConfigFixture): ConfigObject {
  const options = structuredClone(fixtureDocument(document));
  delete options.$schema;
  const entries = options.entries;
  if (
    Object.hasOwn(options, "extends") ||
    (Array.isArray(entries) &&
      entries.some(
        (entry) => entry !== null && typeof entry === "object" && Object.hasOwn(entry, "extends"),
      ))
  ) {
    throw new TypeError("Custom config extends cannot be migrated by a fixture carrier");
  }

  if (Object.hasOwn(options, "vue")) {
    const vue = configObject(options.vue, "Native vue");
    delete options.vue;
    for (const [key, value] of Object.entries(vue)) {
      if (Object.hasOwn(options, key)) {
        throw new TypeError(`Duplicate Vue setting: ${key}`);
      }
      Object.defineProperty(options, key, {
        value,
        enumerable: true,
        configurable: true,
        writable: true,
      });
    }
  }
  if (Object.hasOwn(options, "lsp")) {
    if (Object.hasOwn(options, "languageServer")) {
      throw new TypeError("Duplicate languageServer settings");
    }
    options.languageServer = options.lsp;
    delete options.lsp;
  }
  const typeChecker =
    options.typeChecker === undefined
      ? {}
      : configObject(options.typeChecker, "Native typeChecker");
  options.typeChecker = { jsxTypecheck: false, ...typeChecker };
  return options;
}

/** Preserve source fixtures and emit a separate ordinary Vite configuration. */
export function vueConfigFixtureSource(document: NativeConfigFixture): string {
  const native = fixtureDocument(document);
  const metadata = Object.hasOwn(native, "$schema")
    ? `$schema: ${JSON.stringify(native.$schema)}, `
    : "";
  return `export default { ${metadata}vue: ${JSON.stringify(nativeConfigToVueOptions(native), null, 2)} };\n`;
}

/** Write an active discovery carrier without modifying the original fixture. */
export function writeVueConfigFixture(root: string, document: NativeConfigFixture): string {
  const path = join(root, "vite.config.mjs");
  writeFileSync(path, vueConfigFixtureSource(document));
  return path;
}
