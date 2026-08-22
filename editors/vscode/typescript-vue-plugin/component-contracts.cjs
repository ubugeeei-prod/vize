"use strict";

function vueComponentDisplayParts(ts, sourceText, localName) {
  if (typeof sourceText !== "string") {
    return undefined;
  }

  const script = extractScript(sourceText);
  const lines = [`const ${localName || "component"}: VueComponent`];
  const props = extractMacroType(ts, script, "defineProps");
  const emits = extractMacroType(ts, script, "defineEmits");
  const slots = extractMacroType(ts, script, "defineSlots");
  const model = extractModelContract(ts, script);
  if (props || emits || slots || model) {
    lines.push("{");
    if (props) lines.push(`  props: ${compactType(props)};`);
    if (emits) lines.push(`  emits: ${compactType(emits)};`);
    if (slots) lines.push(`  slots: ${compactType(slots)};`);
    if (model) lines.push(`  model: ${model};`);
    lines.push("}");
  }

  return [{ kind: ts.SymbolDisplayPartKind.text, text: lines.join("\n") }];
}

function extractScript(sourceText) {
  const blocks = [];
  const scriptRe = /<script\b[^>]*>([\s\S]*?)<\/script>/gi;
  for (let match; (match = scriptRe.exec(sourceText));) {
    blocks.push(match[1]);
  }
  return blocks.length > 0 ? blocks.join("\n") : sourceText;
}

function extractMacroType(ts, scriptText, macroName) {
  const call = findMacroCall(ts, scriptText, macroName);
  const typeArgument = call?.typeArguments?.[0];
  return typeArgument ? typeArgument.getText(call.getSourceFile()) : undefined;
}

function extractModelContract(ts, scriptText) {
  const call = findMacroCall(ts, scriptText, "defineModel");
  const typeArgument = call?.typeArguments?.[0];
  if (!typeArgument) return undefined;
  const firstArg = call.arguments?.[0];
  const name = firstArg && isStringLiteralLike(ts, firstArg) ? firstArg.text : "modelValue";
  return `${JSON.stringify(name)}: ${compactType(typeArgument.getText(call.getSourceFile()))}`;
}

function findMacroCall(ts, scriptText, macroName) {
  const sourceFile = ts.createSourceFile(
    "vize-component-contract.ts",
    scriptText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  let result;
  visit(sourceFile, (node) => {
    if (
      !result &&
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === macroName
    ) {
      result = node;
    }
  });
  return result;
}

function visit(node, callback) {
  callback(node);
  node.forEachChild((child) => visit(child, callback));
}

function isStringLiteralLike(ts, node) {
  return ts.isStringLiteral(node) || node.kind === ts.SyntaxKind.NoSubstitutionTemplateLiteral;
}

function compactType(sourceText) {
  let output = "";
  let pendingSpace = false;
  for (const token of lexicalTokens(sourceText)) {
    if (token.kind === "space") {
      pendingSpace = output.length > 0;
      continue;
    }
    if (pendingSpace) {
      output += " ";
    }
    output += token.text;
    pendingSpace = false;
  }
  return output.trim();
}

function lexicalTokens(sourceText) {
  const tokens = [];
  for (let index = 0; index < sourceText.length;) {
    const char = sourceText[index];
    if (/\s/.test(char)) {
      const end = consumeWhile(sourceText, index, (value) => /\s/.test(value));
      tokens.push({ kind: "space", text: sourceText.slice(index, end) });
      index = end;
      continue;
    }
    if ((char === "/" && sourceText[index + 1] === "/") || sourceText.startsWith("/*", index)) {
      index = consumeComment(sourceText, index);
      continue;
    }
    if (char === "'" || char === '"' || char === "`") {
      const end = consumeQuoted(sourceText, index, char);
      tokens.push({ kind: "text", text: sourceText.slice(index, end) });
      index = end;
      continue;
    }
    tokens.push({ kind: "text", text: char });
    index += 1;
  }
  return tokens;
}

function consumeWhile(sourceText, start, predicate) {
  let index = start;
  while (index < sourceText.length && predicate(sourceText[index])) index += 1;
  return index;
}

function consumeComment(sourceText, start) {
  if (sourceText[start + 1] === "/") {
    const end = sourceText.indexOf("\n", start + 2);
    return end < 0 ? sourceText.length : end;
  }
  const end = sourceText.indexOf("*/", start + 2);
  return end < 0 ? sourceText.length : end + 2;
}

function consumeQuoted(sourceText, start, quote) {
  let escaped = false;
  for (let index = start + 1; index < sourceText.length; index += 1) {
    const char = sourceText[index];
    if (escaped) {
      escaped = false;
    } else if (char === "\\") {
      escaped = true;
    } else if (char === quote) {
      return index + 1;
    }
  }
  return sourceText.length;
}

module.exports = { vueComponentDisplayParts };
