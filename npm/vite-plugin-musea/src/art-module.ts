/**
 * Art module generation for Musea.
 *
 * Generates the virtual ES modules that represent parsed `.art.vue` files,
 * including variant component definitions and script setup handling.
 */

import path from "node:path";

import type { ArtFileInfo } from "./types/index.js";
import { toPascalCase } from "./utils.js";

/**
 * Extract the content of the first <script setup> block from a Vue SFC source.
 */
export function extractScriptSetupContent(source: string): string | undefined {
  const match = source.match(/<script\s+[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
  return match?.[1]?.trim();
}

function resolveRelativeSpecifier(specifier: string, artDir: string): string {
  if (!specifier.startsWith(".")) {
    return specifier;
  }

  return path.resolve(artDir, specifier);
}

function rewriteRelativeImportStatement(statement: string, artDir: string): string {
  const rewrittenFromImports = statement.replace(
    /\bfrom\s+(['"])([^'"]+)\1/g,
    (_match, quote: string, specifier: string) =>
      `from ${quote}${resolveRelativeSpecifier(specifier, artDir)}${quote}`,
  );

  return rewrittenFromImports.replace(
    /^(\s*import\s+)(['"])([^'"]+)\2(\s*;?\s*)$/s,
    (_match, prefix: string, quote: string, specifier: string, suffix: string) =>
      `${prefix}${quote}${resolveRelativeSpecifier(specifier, artDir)}${quote}${suffix}`,
  );
}

function countCharBalance(source: string, openChar: string, closeChar: string): number {
  let balance = 0;
  for (const char of source) {
    if (char === openChar) balance++;
    else if (char === closeChar) balance--;
  }
  return balance;
}

function isCompleteImportStatement(statement: string): boolean {
  const trimmed = statement.trim();
  if (!trimmed.startsWith("import ")) {
    return false;
  }

  if (countCharBalance(statement, "{", "}") > 0) {
    return false;
  }

  return (
    /^import\s+[\s\S]+?\s+from\s+['"][^'"]+['"]\s*;?$/s.test(trimmed) ||
    /^import\s+['"][^'"]+['"]\s*;?$/s.test(trimmed)
  );
}

function splitTopLevelCommaList(source: string): string[] {
  const parts: string[] = [];
  let current = "";
  let braceDepth = 0;
  let bracketDepth = 0;
  let parenDepth = 0;

  for (const char of source) {
    if (char === "," && braceDepth === 0 && bracketDepth === 0 && parenDepth === 0) {
      const trimmed = current.trim();
      if (trimmed) {
        parts.push(trimmed);
      }
      current = "";
      continue;
    }

    current += char;

    if (char === "{") braceDepth++;
    else if (char === "}") braceDepth--;
    else if (char === "[") bracketDepth++;
    else if (char === "]") bracketDepth--;
    else if (char === "(") parenDepth++;
    else if (char === ")") parenDepth--;
  }

  const trimmed = current.trim();
  if (trimmed) {
    parts.push(trimmed);
  }

  return parts;
}

function collectImportedNames(statement: string, returnNames: Set<string>): void {
  const normalized = statement.replace(/\s+/g, " ").trim().replace(/;$/, "");
  const fromMatch = normalized.match(/^import\s+(type\s+)?(.+?)\s+from\s+['"][^'"]+['"]$/);

  if (!fromMatch) {
    return;
  }

  if (fromMatch[1]) {
    return;
  }

  const specifiers = fromMatch[2].trim();
  const specifierParts = splitTopLevelCommaList(specifiers);
  const defaultOrNamespace = specifierParts[0]?.trim() ?? "";
  const trailing = specifierParts.slice(1).join(", ").trim();

  if (defaultOrNamespace && !defaultOrNamespace.startsWith("{")) {
    const namespaceMatch = defaultOrNamespace.match(/^\*\s+as\s+([A-Za-z_$][\w$]*)$/);
    if (namespaceMatch) {
      returnNames.add(namespaceMatch[1]);
    } else if (!defaultOrNamespace.startsWith("type ")) {
      returnNames.add(defaultOrNamespace);
    }
  }

  const namedBlock = defaultOrNamespace.startsWith("{")
    ? defaultOrNamespace
    : trailing.startsWith("{")
      ? trailing
      : "";

  if (!namedBlock) {
    return;
  }

  const namedContent = namedBlock.slice(1, -1);
  for (const part of splitTopLevelCommaList(namedContent)) {
    const trimmed = part.trim();
    if (!trimmed || trimmed.startsWith("type ")) {
      continue;
    }

    const alias = trimmed
      .split(/\s+as\s+/)
      .pop()
      ?.trim();
    if (alias) {
      returnNames.add(alias);
    }
  }
}

function collectObjectDestructuredNames(statement: string, returnNames: Set<string>): void {
  const match = statement.match(/^(?:export\s+)?(?:const|let|var)\s+\{([\s\S]*?)\}\s*=/);
  if (!match) {
    return;
  }

  for (const part of splitTopLevelCommaList(match[1])) {
    let name = part.trim();
    if (!name) {
      continue;
    }

    if (name.startsWith("...")) {
      name = name.slice(3).trim();
    } else if (name.includes(":")) {
      name = name.split(":").pop()!.trim();
    } else if (name.includes("=")) {
      name = name.split("=")[0].trim();
    }

    if (name.includes("=")) {
      name = name.split("=")[0].trim();
    }

    if (/^[A-Za-z_$][\w$]*$/.test(name)) {
      returnNames.add(name);
    }
  }
}

function collectArrayDestructuredNames(statement: string, returnNames: Set<string>): void {
  const match = statement.match(/^(?:export\s+)?(?:const|let|var)\s+\[([\s\S]*?)\]\s*=/);
  if (!match) {
    return;
  }

  for (const part of splitTopLevelCommaList(match[1])) {
    let name = part.trim();
    if (!name) {
      continue;
    }

    if (name.startsWith("...")) {
      name = name.slice(3).trim();
    }

    if (name.includes("=")) {
      name = name.split("=")[0].trim();
    }

    if (/^[A-Za-z_$][\w$]*$/.test(name)) {
      returnNames.add(name);
    }
  }
}

function collectTopLevelReturnNames(setupBody: string[], returnNames: Set<string>): void {
  let braceDepth = 0;

  for (let i = 0; i < setupBody.length; i++) {
    const line = setupBody[i];
    const trimmed = line.trim();

    if (braceDepth === 0) {
      if (/^(?:export\s+)?(?:const|let|var)\s+\{/.test(trimmed)) {
        const statementLines = [line];
        let balance =
          countCharBalance(line, "{", "}") +
          countCharBalance(line, "[", "]") +
          countCharBalance(line, "(", ")");

        while (balance > 0 && i + 1 < setupBody.length) {
          i++;
          statementLines.push(setupBody[i]);
          balance +=
            countCharBalance(setupBody[i], "{", "}") +
            countCharBalance(setupBody[i], "[", "]") +
            countCharBalance(setupBody[i], "(", ")");
        }

        collectObjectDestructuredNames(statementLines.join("\n"), returnNames);
        continue;
      }

      if (/^(?:export\s+)?(?:const|let|var)\s+\[/.test(trimmed)) {
        const statementLines = [line];
        let balance =
          countCharBalance(line, "{", "}") +
          countCharBalance(line, "[", "]") +
          countCharBalance(line, "(", ")");

        while (balance > 0 && i + 1 < setupBody.length) {
          i++;
          statementLines.push(setupBody[i]);
          balance +=
            countCharBalance(setupBody[i], "{", "}") +
            countCharBalance(setupBody[i], "[", "]") +
            countCharBalance(setupBody[i], "(", ")");
        }

        collectArrayDestructuredNames(statementLines.join("\n"), returnNames);
        continue;
      }

      const constMatch = trimmed.match(/^(?:export\s+)?(?:const|let|var)\s+([A-Za-z_$][\w$]*)/);
      if (constMatch) {
        returnNames.add(constMatch[1]);
      }

      const functionMatch = trimmed.match(
        /^(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][\w$]*)\s*\(/,
      );
      if (functionMatch) {
        returnNames.add(functionMatch[1]);
      }

      const classMatch = trimmed.match(/^(?:export\s+)?class\s+([A-Za-z_$][\w$]*)\b/);
      if (classMatch) {
        returnNames.add(classMatch[1]);
      }
    }

    braceDepth += countCharBalance(line, "{", "}");
  }
}

/**
 * Parse script setup content into imports and setup body.
 * Returns the import lines, setup body lines, and all identifiers to expose.
 */
export function parseScriptSetupForArt(content: string): {
  imports: string[];
  setupBody: string[];
  returnNames: string[];
} {
  const lines = content.split("\n");
  const imports: string[] = [];
  const setupBody: string[] = [];
  const returnNames: Set<string> = new Set();
  let currentImport: string[] | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    if (currentImport) {
      currentImport.push(line);
      const statement = currentImport.join("\n");
      if (isCompleteImportStatement(statement)) {
        imports.push(statement);
        collectImportedNames(statement, returnNames);
        currentImport = null;
      }
      continue;
    }

    if (trimmed.startsWith("import ")) {
      currentImport = [line];
      const statement = currentImport.join("\n");
      if (isCompleteImportStatement(statement)) {
        imports.push(statement);
        collectImportedNames(statement, returnNames);
        currentImport = null;
      }
      continue;
    }

    setupBody.push(line);
  }

  if (currentImport) {
    const statement = currentImport.join("\n");
    imports.push(statement);
    collectImportedNames(statement, returnNames);
  }

  collectTopLevelReturnNames(setupBody, returnNames);

  // Remove 'type' keyword imports and common Vue utilities that shouldn't be returned
  returnNames.delete("type");

  return {
    imports,
    setupBody,
    returnNames: [...returnNames],
  };
}

export function generateArtModule(art: ArtFileInfo, filePath: string): string {
  let componentImportPath: string | undefined;
  let componentName: string | undefined;

  if (art.isInline && art.componentPath) {
    // Inline art: import the host .vue file itself as the component
    componentImportPath = art.componentPath;
    componentName = path.basename(art.componentPath, ".vue");
  } else if (art.metadata.component) {
    // Traditional .art.vue: resolve component from the component attribute
    const comp = art.metadata.component;
    componentImportPath = path.isAbsolute(comp) ? comp : path.resolve(path.dirname(filePath), comp);
    componentName = path.basename(comp, ".vue");
  }

  // Parse script setup if present
  const scriptSetup = art.scriptSetupContent
    ? parseScriptSetupForArt(art.scriptSetupContent)
    : null;

  let code = `
// Auto-generated module for: ${path.basename(filePath)}
import { defineComponent, h } from 'vue';
`;

  // Add script setup imports at module level
  // Resolve relative paths to absolute since this code runs inside a virtual module
  if (scriptSetup) {
    const artDir = path.dirname(filePath);
    for (const imp of scriptSetup.imports) {
      const resolved = rewriteRelativeImportStatement(imp, artDir);
      code += `${resolved}\n`;
    }
  }

  if (componentImportPath && componentName) {
    // Only add component import if not already imported by script setup
    const alreadyImported = scriptSetup?.imports.some((imp) => {
      // Check against the original relative path and the resolved absolute path
      if (
        imp.includes(`from '${componentImportPath}'`) ||
        imp.includes(`from "${componentImportPath}"`)
      )
        return true;
      // Also check by component name as default import (handles relative vs absolute path mismatch)
      return new RegExp(`^import\\s+${componentName}[\\s,]`).test(imp.trim());
    });
    if (!alreadyImported) {
      code += `import ${componentName} from '${componentImportPath}';\n`;
    }
    code += `export const __component__ = ${componentName};\n`;
  }

  code += `
export const metadata = ${JSON.stringify(art.metadata)};
export const variants = ${JSON.stringify(art.variants)};
export const __styles__ = ${JSON.stringify(art.styleBlocks ?? [])};
`;

  // Generate variant components
  for (const variant of art.variants) {
    const variantComponentName = toPascalCase(variant.name);

    let template = variant.template;

    // Replace <Self> with the actual component name (for inline art)
    if (componentName) {
      template = template
        .replace(/<Self/g, `<${componentName}`)
        .replace(/<\/Self>/g, `</${componentName}>`);
    }

    // Escape the template for use in a JS string
    const escapedTemplate = template
      .replace(/\\/g, "\\\\")
      .replace(/`/g, "\\`")
      .replace(/\$/g, "\\$");

    // Wrap template with the variant container (no .musea-variant class -- the
    // outer mount container already carries it; duplicating causes double padding)
    const fullTemplate = `<div data-variant="${variant.name}">${escapedTemplate}</div>`;

    // Collect component names for the `components` option.
    // Runtime-compiled templates use resolveComponent() which checks the
    // `components` option, NOT setup return values.
    const componentNames = new Set<string>();
    if (componentName) componentNames.add(componentName);
    if (scriptSetup) {
      for (const name of scriptSetup.returnNames) {
        // PascalCase names starting with uppercase are likely components
        if (/^[A-Z]/.test(name)) componentNames.add(name);
      }
    }
    const components =
      componentNames.size > 0 ? `  components: { ${[...componentNames].join(", ")} },\n` : "";

    const hasSetupBody = scriptSetup?.setupBody.some((line) => line.trim().length > 0) ?? false;

    if (scriptSetup && (hasSetupBody || scriptSetup.returnNames.length > 0)) {
      // Generate variant with setup function from art file's <script setup>
      code += `
export const ${variantComponentName} = defineComponent({
  name: '${variantComponentName}',
${components}  setup() {
${scriptSetup.setupBody.map((l) => `    ${l}`).join("\n")}
    return { ${scriptSetup.returnNames.join(", ")} };
  },
  template: \`${fullTemplate}\`,
});
`;
    } else if (componentName) {
      code += `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
${components}  template: \`${fullTemplate}\`,
};
`;
    } else {
      code += `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
  template: \`${fullTemplate}\`,
};
`;
    }
  }

  // Default export
  const defaultVariant = art.variants.find((v) => v.isDefault) || art.variants[0];
  if (defaultVariant) {
    code += `
export default ${toPascalCase(defaultVariant.name)};
`;
  }

  return code;
}
