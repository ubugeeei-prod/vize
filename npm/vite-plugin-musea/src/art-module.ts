/**
 * Art module generation for Musea.
 *
 * Generates the virtual ES modules that represent parsed `.art.vue` files,
 * including variant component definitions and script setup handling.
 */

import path from "node:path";

import { allowedSourceRoots, resolveComponentSourcePath } from "./component-source.js";
import type { ArtFileInfo } from "./types/index.js";
import { escapeHtml, toPascalCase } from "./utils.js";

/**
 * Extract the content of the first <script setup> block from a Vue SFC source.
 */
export function extractScriptSetupContent(source: string): string | undefined {
  const match = source.match(/<script\s+[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
  return match?.[1]?.trim();
}

export function extractScriptSetupIsolated(source: string): boolean {
  const match = source.match(/<script\s+([^>]*)\bsetup\b([^>]*)>/);
  if (!match) return true;
  const attrs = `${match[1] ?? ""} ${match[2] ?? ""}`;
  const isolate = attrs.match(/\bisolate\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/);
  return (isolate?.[1] ?? isolate?.[2] ?? isolate?.[3]) !== "false";
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

function escapeTemplateLiteral(str: string): string {
  return str.replace(/\\/g, "\\\\").replace(/`/g, "\\`").replace(/\$/g, "\\$");
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

function importDeclaresName(statement: string, name: string): boolean {
  const names = new Set<string>();
  collectImportedNames(statement, names);
  return names.has(name);
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
  defineArtComponentName?: string;
  defineArtComponentSource?: string;
} {
  const lines = content.split("\n");
  const imports: string[] = [];
  const setupBody: string[] = [];
  const returnNames: Set<string> = new Set();
  let currentImport: string[] | null = null;
  const defineArtComponent = extractDefineArtComponent(content);
  let defineArtBalance = 0;

  for (const line of lines) {
    const trimmed = line.trim();

    if (defineArtBalance > 0) {
      defineArtBalance += countCharBalance(line, "(", ")");
      continue;
    }

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

    if (isDefineArtLine(trimmed)) {
      defineArtBalance = Math.max(0, countCharBalance(line, "(", ")"));
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
  if (defineArtComponent.name) {
    returnNames.delete(defineArtComponent.name);
  }

  return {
    imports,
    setupBody,
    returnNames: [...returnNames],
    defineArtComponentName: defineArtComponent.name,
    defineArtComponentSource: defineArtComponent.source,
  };
}

function extractDefineArtComponent(content: string): { name?: string; source?: string } {
  const sourceMatch = content.match(/\bdefineArt\s*\(\s*(['"])([^'"]+)\1/);
  if (sourceMatch) {
    return {
      name: componentNameFromSource(sourceMatch[2]),
      source: sourceMatch[2],
    };
  }

  const identifierMatch = content.match(/\bdefineArt\s*\(\s*([A-Za-z_$][\w$]*)/);
  return { name: identifierMatch?.[1] };
}

function componentNameFromSource(source: string): string {
  const withoutQuery = source.split(/[?#]/, 1)[0] || source;
  const filename = path.basename(withoutQuery);
  const extension = path.extname(filename);
  const stem = extension ? filename.slice(0, -extension.length) : filename;
  const name = toPascalCase(stem);
  return name === "Variant" ? "MuseaComponent" : name;
}

function isDefineArtLine(trimmed: string): boolean {
  return /\bdefineArt\s*\(/.test(trimmed);
}

interface GenerateArtModuleOptions {
  root?: string;
  scanRoots?: string[];
}

export function generateArtModule(
  art: ArtFileInfo,
  filePath: string,
  options: GenerateArtModuleOptions = {},
): string {
  let componentImportPath: string | undefined;
  let componentTagName: string | undefined;
  let componentBindingName = "__MuseaComponent";
  const scriptSetup = art.scriptSetupContent
    ? parseScriptSetupForArt(art.scriptSetupContent)
    : null;
  const defineArtComponentName = scriptSetup?.defineArtComponentName;
  const defineArtComponentSource = scriptSetup?.defineArtComponentSource;

  if (art.isInline && art.componentPath) {
    // Inline art: import the host .vue file itself as the component
    componentImportPath = options.root
      ? (resolveComponentSourcePath(
          art,
          filePath,
          allowedSourceRoots(options.root, options.scanRoots ?? []),
        ) ?? undefined)
      : art.componentPath;
    componentTagName = "MuseaComponent";
  } else if (defineArtComponentSource || art.metadata.component) {
    // .art.vue: resolve component from defineArt(source, ...) or the legacy component attribute.
    const componentSource = defineArtComponentSource ?? art.metadata.component;
    if (componentSource) {
      const sourceArt =
        componentSource === art.metadata.component
          ? art
          : { ...art, metadata: { ...art.metadata, component: componentSource } };
      componentImportPath = options.root
        ? (resolveComponentSourcePath(
            sourceArt,
            filePath,
            allowedSourceRoots(options.root, options.scanRoots ?? []),
          ) ?? undefined)
        : path.isAbsolute(componentSource)
          ? componentSource
          : path.resolve(path.dirname(filePath), componentSource);
    }
    componentTagName =
      defineArtComponentName ??
      (art.metadata.component ? componentNameFromSource(art.metadata.component) : "MuseaComponent");
    componentBindingName = componentTagName;
  }

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

  if (componentImportPath && componentTagName) {
    // Only add component import if not already imported by script setup
    const alreadyImported = scriptSetup?.imports.some((imp) =>
      importDeclaresName(imp, componentBindingName),
    );
    if (!alreadyImported) {
      code += `import ${componentBindingName} from ${JSON.stringify(componentImportPath)};\n`;
    }
    code += `export const __component__ = ${componentBindingName};\n`;
  }

  code += `
export const metadata = ${JSON.stringify(art.metadata)};
export const variants = ${JSON.stringify(art.variants)};
export const __styles__ = ${JSON.stringify(art.styleBlocks ?? [])};
`;

  const hasSetupBody = scriptSetup?.setupBody.some((line) => line.trim().length > 0) ?? false;
  const hasSetup = !!scriptSetup && (hasSetupBody || scriptSetup.returnNames.length > 0);
  const setupReturn = `{ ${scriptSetup?.returnNames.join(", ") ?? ""} }`;
  const isolatedSetup = art.scriptSetupIsolated !== false;

  if (scriptSetup && hasSetup && !isolatedSetup) {
    code += `
const __museaSharedSetup = (() => {
${scriptSetup.setupBody.map((l) => `  ${l}`).join("\n")}
  return ${setupReturn};
})();
`;
  }

  // Generate variant components
  for (const variant of art.variants) {
    const variantComponentName = toPascalCase(variant.name);

    let template = variant.template;

    // Replace <Self> with the actual component name (for inline art)
    if (componentTagName) {
      template = template
        .replace(/<Self/g, `<${componentTagName}`)
        .replace(/<\/Self>/g, `</${componentTagName}>`);
    }

    // Escape the template for use in a JS string
    const escapedTemplate = escapeTemplateLiteral(template);
    const escapedVariantName = escapeTemplateLiteral(escapeHtml(variant.name));

    // Wrap template with the variant container (no .musea-variant class -- the
    // outer mount container already carries it; duplicating causes double padding)
    const fullTemplate = `<div data-variant="${escapedVariantName}">${escapedTemplate}</div>`;

    // Collect component names for the `components` option.
    // Runtime-compiled templates use resolveComponent() which checks the
    // `components` option, NOT setup return values.
    const componentNames = new Map<string, string>();
    if (componentTagName) componentNames.set(componentTagName, componentBindingName);
    if (scriptSetup) {
      for (const name of scriptSetup.returnNames) {
        // PascalCase names starting with uppercase are likely components
        if (/^[A-Z]/.test(name)) componentNames.set(name, name);
      }
    }
    const components =
      componentNames.size > 0
        ? `  components: { ${[...componentNames]
            .map(([name, value]) => `${JSON.stringify(name)}: ${value}`)
            .join(", ")} },\n`
        : "";

    if (scriptSetup && hasSetup && isolatedSetup) {
      // Generate variant with setup function from art file's <script setup>
      code += `
export const ${variantComponentName} = defineComponent({
  name: '${variantComponentName}',
${components}  setup() {
${scriptSetup.setupBody.map((l) => `    ${l}`).join("\n")}
    return ${setupReturn};
  },
  template: \`${fullTemplate}\`,
});
`;
    } else if (scriptSetup && hasSetup) {
      code += `
export const ${variantComponentName} = defineComponent({
  name: '${variantComponentName}',
${components}  setup() {
    return __museaSharedSetup;
  },
  template: \`${fullTemplate}\`,
});
`;
    } else if (componentTagName) {
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
