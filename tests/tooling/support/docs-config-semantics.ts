import assert from "node:assert/strict";
import { Script } from "node:vm";

type Options = Record<string, unknown>;
type Environment = { command: string; mode: string; isSsrBuild: boolean };

/** Evaluate only the finite, trusted config recipes committed in test fixtures. */
export function configRecipe(source: string, environment: Environment): Options {
  if (source.trim().startsWith("{")) return JSON.parse(source) as Options;
  const defineConfig = (input: Options | ((env: Environment) => Options)): Options => {
    const config = typeof input === "function" ? input(environment) : input;
    Object.defineProperty(config, "withVue", {
      value: (vue: Options) => ({ ...config, vue }),
    });
    return config;
  };
  const expression = source.replace(/^import .*;\n/gm, "").replace("export default ", "return ");
  const script = new Script(`(() => {\n${expression}\n})()`, {
    filename: "archived-doc-config.js",
  });
  const result = script.runInNewContext(
    {
      defineConfig,
      musea: (options: Options) => ({ musea: options }),
      vize: (options: Options) => ({ compiler: options }),
    },
    { timeout: 1_000 },
  ) as Options;
  // Normalize cross-realm prototypes without discarding RegExp option values.
  return structuredClone(result);
}

export function nativeOptions(config: Options): Options {
  const options = structuredClone(config);
  delete options.$schema;
  if (Object.hasOwn(options, "lsp")) {
    options.languageServer = options.lsp;
    delete options.lsp;
  }
  return options;
}

/** Consolidated examples may include another recipe's options, never lose one. */
export function includesOptions(actual: unknown, expected: unknown, label = "config"): void {
  if (
    expected !== null &&
    typeof expected === "object" &&
    !Array.isArray(expected) &&
    !(expected instanceof RegExp)
  ) {
    assert.ok(actual !== null && typeof actual === "object", label);
    for (const [key, value] of Object.entries(expected)) {
      assert.ok(Object.hasOwn(actual, key), `${label}.${key}: option retained`);
      includesOptions((actual as Options)[key], value, `${label}.${key}`);
    }
  } else {
    assert.deepEqual(actual, expected, label);
  }
}

/** Decode this archived PKL recipe's finite scalar fields, not a PKL runtime. */
export function retiredPklOptions(source: string): Options {
  const block = (name: string) => {
    const start = source.indexOf(`${name} {`);
    assert.ok(start >= 0, name);
    let depth = 1;
    const bodyStart = start + name.length + 2;
    let end = bodyStart;
    for (; depth && end < source.length; end++) {
      if (source[end] === "{") depth++;
      if (source[end] === "}") depth--;
    }
    assert.equal(depth, 0, name);
    return source.slice(bodyStart, end - 1);
  };
  const scalars = (text: string) =>
    Object.fromEntries(
      [...text.matchAll(/(\w+)\s*=\s*(true|false|"[^"\n]*")/g)].map(([, key, value]) => [
        key,
        JSON.parse(value),
      ]),
    );
  const entry = source.slice(source.indexOf("new ConfigEntry {"));
  const strings = (text: string) => [...text.matchAll(/"([^"\n]*)"/g)].map(([, value]) => value);
  return {
    compiler: scalars(block("compiler")),
    vite: { scanPatterns: strings(block("vite")) },
    linter: scalars(block("linter")),
    typeChecker: scalars(block("typeChecker")),
    languageServer: scalars(block("lsp")),
    entries: [
      {
        name: scalars(entry).name,
        basePath: scalars(entry).basePath,
        files: strings(entry.match(/files = new Listing \{([^}]+)\}/)?.[1] ?? ""),
        typeChecker: { tsconfig: scalars(entry).tsconfig },
      },
    ],
  };
}
