/** Differential runner for @nuxt/eslint-plugin's nuxt-config-keys-order rule. */
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

function offsetFromLocation(source, line, column) {
  let offset = 0;
  for (let current = 1; current < line; current++) {
    const newline = source.indexOf("\n", offset);
    if (newline === -1) throw new Error(`line ${line} is outside the recorded source`);
    offset = newline + 1;
  }
  return offset + column - 1;
}

function recordMessage(source, message) {
  if (!message.fix) {
    throw new Error(`nuxt/nuxt-config-keys-order emitted a non-fixable diagnostic`);
  }
  return {
    ruleId: message.ruleId,
    severity: message.severity,
    message: message.message,
    line: message.line,
    column: message.column,
    endLine: message.endLine,
    endColumn: message.endColumn,
    range: [
      offsetFromLocation(source, message.line, message.column),
      offsetFromLocation(source, message.endLine, message.endColumn),
    ],
    fix: { range: [...message.fix.range], text: message.fix.text },
  };
}

/** Run the real rule through diagnostics, complete fixing, and idempotence. */
export async function recordNuxtConfigKeysOrderCases(moduleEntry, corpus, packageVersionFrom) {
  const requireFromNuxt = createRequire(fileURLToPath(moduleEntry));
  const pluginEntry = requireFromNuxt.resolve("@nuxt/eslint-plugin");
  const [eslint, pluginModule] = await Promise.all([
    import(requireFromNuxt.resolve("eslint")),
    import(pluginEntry),
  ]);
  const linter = new eslint.Linter({ configType: "flat" });
  const config = {
    languageOptions: { ecmaVersion: "latest", sourceType: "module" },
    plugins: { nuxt: pluginModule.default },
    rules: { "nuxt/nuxt-config-keys-order": "error" },
  };

  const recorded = {};
  for (const entry of corpus.nuxtConfigKeysOrderCases) {
    const messages = linter
      .verify(entry.source, config)
      .map((message) => recordMessage(entry.source, message));
    const firstPass = linter.verifyAndFix(entry.source, config);
    const secondPass = linter.verifyAndFix(firstPass.output, config);
    recorded[entry.id] = {
      messages,
      output: firstPass.output,
      fixed: firstPass.fixed,
      secondPassMessageCount: secondPass.messages.length,
      secondPassOutput: secondPass.output,
      secondPassFixed: secondPass.fixed,
    };
  }
  return {
    cases: recorded,
    pluginVersion: packageVersionFrom(pathToFileURL(pluginEntry)),
  };
}
