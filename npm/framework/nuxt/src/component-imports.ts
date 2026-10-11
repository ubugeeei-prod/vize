import type { NuxtComponentImport } from "./components.ts";

const COMPONENT_CALL_RE = /_?resolveComponent\s*\(\s*["'`]([^"'`]+)["'`]\s*(?:,\s*[^)]+)?\)/g;
const COMPONENTS_IMPORT_RE = /import\s+(?!type\b)\{([^}]*)\}\s+from\s+(["'])#components\2\s*;?/g;
const NUXT_ROUTE_ANNOUNCER_RE = /(?:^|[/\\])nuxt-route-announcer\.(?:[cm]?js|ts|vue)$/;
const IMPORT_SPECIFIER_RE = /^(type\s+)?([A-Za-z_$][\w$]*)(?:\s+as\s+([A-Za-z_$][\w$]*))?$/;

interface ComponentImportSpecifier {
  importedName: string;
  localName: string;
  typeOnly: boolean;
}

interface ComponentBindingResult {
  needsCreateClientOnly: boolean;
  needsDefineAsyncComponent: boolean;
}

function needsClientOnlyWrapper(resolved: NuxtComponentImport): boolean {
  if (resolved.mode !== "client") {
    return false;
  }

  // NuxtRouteAnnouncer supplies scoped default slot props itself. The generic
  // client-only wrapper can expose a props-less placeholder slot path during
  // hydration, which breaks destructured slots like `v-slot="{ message }"`.
  return !NUXT_ROUTE_ANNOUNCER_RE.test(resolved.filePath);
}

function parseComponentImportSpecifier(raw: string): ComponentImportSpecifier | null {
  const trimmed = raw.trim();
  if (!trimmed) {
    return null;
  }

  const match = trimmed.match(IMPORT_SPECIFIER_RE);
  if (!match) {
    return null;
  }

  const [, typeKeyword, importedName, localName] = match;
  return {
    importedName,
    localName: localName || importedName,
    typeOnly: Boolean(typeKeyword),
  };
}

function splitComponentImportSpecifiers(specifiers: string): string[] {
  return specifiers
    .split(",")
    .map((specifier) => specifier.trim())
    .filter(Boolean);
}

function addResolvedComponentBinding(
  componentImports: string[],
  resolved: NuxtComponentImport,
  variableName: string,
  rawVariableName: string,
): ComponentBindingResult {
  let needsCreateClientOnly = false;
  let needsDefineAsyncComponent = false;
  const wrapClientOnly = needsClientOnlyWrapper(resolved);

  if (resolved.lazy) {
    needsDefineAsyncComponent = true;
    const exportAccessor =
      resolved.exportName === "default"
        ? "module.default"
        : `module[${JSON.stringify(resolved.exportName)}]`;
    if (wrapClientOnly) {
      needsCreateClientOnly = true;
      componentImports.push(
        `const ${variableName} = __nuxt_define_async_component(() => import(${JSON.stringify(resolved.filePath)}).then((module) => __nuxt_create_client_only(${exportAccessor})));`,
      );
    } else {
      componentImports.push(
        `const ${variableName} = __nuxt_define_async_component(() => import(${JSON.stringify(resolved.filePath)}).then((module) => ${exportAccessor}));`,
      );
    }
    return { needsCreateClientOnly, needsDefineAsyncComponent };
  }

  if (resolved.exportName === "default") {
    if (wrapClientOnly) {
      needsCreateClientOnly = true;
      componentImports.push(`import ${rawVariableName} from ${JSON.stringify(resolved.filePath)};`);
      componentImports.push(
        `const ${variableName} = __nuxt_create_client_only(${rawVariableName});`,
      );
    } else {
      componentImports.push(`import ${variableName} from ${JSON.stringify(resolved.filePath)};`);
    }
    return { needsCreateClientOnly, needsDefineAsyncComponent };
  }

  if (wrapClientOnly) {
    needsCreateClientOnly = true;
    componentImports.push(
      `import { ${resolved.exportName} as ${rawVariableName} } from ${JSON.stringify(resolved.filePath)};`,
    );
    componentImports.push(`const ${variableName} = __nuxt_create_client_only(${rawVariableName});`);
  } else {
    componentImports.push(
      `import { ${resolved.exportName} as ${variableName} } from ${JSON.stringify(resolved.filePath)};`,
    );
  }

  return { needsCreateClientOnly, needsDefineAsyncComponent };
}

export function injectNuxtComponentImports(
  code: string,
  resolveComponentImport: (name: string) => NuxtComponentImport | null,
): string {
  const componentImports: string[] = [];
  const importedComponents = new Map<string, string>();
  let counter = 0;
  let importCounter = 0;
  let needsDefineAsyncComponent = false;
  let needsCreateClientOnly = false;

  const codeWithComponentImports = code.replace(
    COMPONENTS_IMPORT_RE,
    (match: string, specifiers: string) => {
      const explicitComponentImports: string[] = [];
      const unresolvedSpecifiers: string[] = [];
      let changed = false;

      for (const rawSpecifier of splitComponentImportSpecifiers(specifiers)) {
        const specifier = parseComponentImportSpecifier(rawSpecifier);
        if (!specifier || specifier.typeOnly) {
          unresolvedSpecifiers.push(rawSpecifier);
          continue;
        }

        const resolved = resolveComponentImport(specifier.importedName);
        if (!resolved) {
          unresolvedSpecifiers.push(rawSpecifier);
          continue;
        }

        changed = true;
        const result = addResolvedComponentBinding(
          explicitComponentImports,
          resolved,
          specifier.localName,
          `__nuxt_import_component_${importCounter++}_raw`,
        );
        needsCreateClientOnly ||= result.needsCreateClientOnly;
        needsDefineAsyncComponent ||= result.needsDefineAsyncComponent;
      }

      if (!changed) {
        return match;
      }

      // Keep explicit component dependencies at their authored import position.
      // Only template resolveComponent bindings belong in the leading preamble.
      if (unresolvedSpecifiers.length > 0) {
        explicitComponentImports.push(
          `import { ${unresolvedSpecifiers.join(", ")} } from "#components";`,
        );
      }
      return explicitComponentImports.join("\n");
    },
  );

  const nextCode = codeWithComponentImports.replace(
    COMPONENT_CALL_RE,
    (match: string, name: string) => {
      const resolved = resolveComponentImport(name);
      if (!resolved) {
        return match;
      }

      const importKey = `${resolved.exportName}\u0000${resolved.filePath}\u0000${resolved.lazy ? "lazy" : "eager"}\u0000${resolved.mode ?? "default"}`;
      let variableName = importedComponents.get(importKey);
      if (!variableName) {
        variableName = `__nuxt_component_${counter++}`;
        importedComponents.set(importKey, variableName);
        const result = addResolvedComponentBinding(
          componentImports,
          resolved,
          variableName,
          `${variableName}_raw`,
        );
        needsCreateClientOnly ||= result.needsCreateClientOnly;
        needsDefineAsyncComponent ||= result.needsDefineAsyncComponent;
      }

      return variableName;
    },
  );

  if (componentImports.length === 0 && !needsDefineAsyncComponent && !needsCreateClientOnly) {
    return codeWithComponentImports;
  }

  const preamble = [
    ...(needsDefineAsyncComponent
      ? ['import { defineAsyncComponent as __nuxt_define_async_component } from "vue";']
      : []),
    ...(needsCreateClientOnly
      ? [
          'import { createClientOnly as __nuxt_create_client_only } from "#app/components/client-only";',
        ]
      : []),
    ...componentImports,
  ];

  return preamble.join("\n") + "\n" + nextCode;
}
