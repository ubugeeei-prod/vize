/** Differential runner for @nuxt/eslint-plugin's no-page-meta-runtime-values rule. */
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

function offsetFromLocation(source, line, column) {
  let offset = 0;
  let currentLine = 1;
  while (currentLine < line) {
    const newline = source.indexOf("\n", offset);
    if (newline === -1) throw new Error(`line ${line} is outside the recorded source`);
    offset = newline + 1;
    currentLine++;
  }
  return offset + column - 1;
}

/** Reduce one ESLint diagnostic to the exact non-fixable Patina contract. */
function recordPageMetaMessage(source, message) {
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
    fix: message.fix ? { range: [...message.fix.range], text: message.fix.text } : null,
  };
}

/** Run the real rule twice, recording its non-fixable stability. */
export async function recordNoPageMetaRuntimeValuesCases(moduleEntry, corpus, packageVersionFrom) {
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
    rules: { "nuxt/no-page-meta-runtime-values": "error" },
  };

  const recorded = {};
  for (const entry of corpus.noPageMetaRuntimeValuesCases) {
    const messages = linter
      .verify(entry.source, config)
      .map((message) => recordPageMetaMessage(entry.source, message));
    const firstPass = linter.verifyAndFix(entry.source, config);
    const secondPass = linter.verifyAndFix(firstPass.output, config);
    const secondPassMessages = secondPass.messages.map((message) =>
      recordPageMetaMessage(secondPass.output, message),
    );
    recorded[entry.id] = {
      messages,
      output: firstPass.output,
      fixed: firstPass.fixed,
      secondPassMessageCount: secondPassMessages.length,
      secondPassMessagesMatch: JSON.stringify(secondPassMessages) === JSON.stringify(messages),
      secondPassOutput: secondPass.output,
      secondPassFixed: secondPass.fixed,
    };
  }
  return {
    cases: recorded,
    pluginVersion: packageVersionFrom(pathToFileURL(pluginEntry)),
  };
}
