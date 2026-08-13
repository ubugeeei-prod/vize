const probeName = "__vize_typecheck_mutation_probe";
export const seededMutationDiagnostic = {
  severity: "error",
  column: 1,
  code: 2322,
  message: "Type 'number' is not assignable to type 'string'.",
};
const probeStatement = `const ${probeName}: string = 1`;

export function buildSeededMutation(cleanSource) {
  const script = findScriptBlocks(cleanSource);
  const setupTs = script.find((block) => block.setup && block.typescript && block.closeIndex >= 0);
  if (setupTs != null) {
    const beforeClose = cleanSource.slice(0, setupTs.closeIndex);
    const separator = beforeClose.endsWith("\n") ? "" : "\n";
    const prefix = `${beforeClose}${separator}`;
    return {
      brokenSource: `${prefix}${probeStatement}\n${cleanSource.slice(setupTs.closeIndex)}`,
      line: lineCount(prefix),
      column: seededMutationDiagnostic.column,
    };
  }

  const hasSetup = script.some((block) => block.setup);
  if (!hasSetup) return appendedMutation(cleanSource, "script setup");
  const hasNormalScript = script.some((block) => !block.setup);
  return hasNormalScript ? null : appendedMutation(cleanSource, "script");
}

function findScriptBlocks(source) {
  const blocks = [];
  // A commented-out opening tag would otherwise win the block selection and the
  // probe would land inside the comment, where neither typechecker compiles it.
  // The mask preserves length, so indices stay aligned with `source`.
  const scannable = source.replaceAll(/<!--[\s\S]*?-->/g, (comment) => " ".repeat(comment.length));
  const pattern = /<script\b([^>]*)>/gi;
  let match;
  while ((match = pattern.exec(scannable)) != null) {
    blocks.push({
      setup: /\bsetup(?:\s|=|>|$)/i.test(match[1]),
      typescript: /\blang\s*=\s*(?:"tsx?"|'tsx?'|tsx?)(?:\s|>|$)/i.test(match[1]),
      closeIndex: source.indexOf("</script>", pattern.lastIndex),
    });
  }
  return blocks;
}

function appendedMutation(cleanSource, tag) {
  const prefix = cleanSource.endsWith("\n") ? cleanSource : `${cleanSource}\n`;
  const beforeProbe = `${prefix}<${tag} lang="ts">\n`;
  return {
    brokenSource: `${beforeProbe}${probeStatement}\n</script>\n`,
    line: lineCount(beforeProbe),
    column: seededMutationDiagnostic.column,
  };
}

function lineCount(value) {
  return value.replaceAll("\r\n", "\n").split("\n").length;
}
