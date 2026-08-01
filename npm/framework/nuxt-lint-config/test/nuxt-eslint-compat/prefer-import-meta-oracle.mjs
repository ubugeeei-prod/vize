/** Differential runner for @nuxt/eslint-plugin's prefer-import-meta rule. */
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

/** Reduce one ESLint diagnostic to the exact contract the Patina rule shares. */
function recordLintMessage(message) {
  if (!message.fix) {
    throw new Error(`nuxt/prefer-import-meta emitted a non-fixable diagnostic: ${message.message}`);
  }
  return {
    ruleId: message.ruleId,
    severity: message.severity,
    message: message.message,
    line: message.line,
    column: message.column,
    endLine: message.endLine,
    endColumn: message.endColumn,
    fix: { range: [...message.fix.range], text: message.fix.text },
  };
}

/** Run the real rule, including a second fix pass proving convergence. */
export async function recordPreferImportMetaCases(moduleEntry, corpus, packageVersionFrom) {
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
    rules: { "nuxt/prefer-import-meta": "error" },
  };

  const recorded = {};
  for (const entry of corpus.preferImportMetaCases) {
    const messages = linter.verify(entry.source, config).map(recordLintMessage);
    const firstPass = linter.verifyAndFix(entry.source, config);
    const secondPass = linter.verifyAndFix(firstPass.output, config);
    recorded[entry.id] = {
      messages,
      output: firstPass.output,
      fixed: firstPass.fixed,
      secondPassMessages: secondPass.messages,
      secondPassOutput: secondPass.output,
      secondPassFixed: secondPass.fixed,
    };
  }
  return {
    cases: recorded,
    pluginVersion: packageVersionFrom(pathToFileURL(pluginEntry)),
  };
}
