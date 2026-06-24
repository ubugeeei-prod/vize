import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

export interface NuxtComponentDescriptor {
  pascalName?: string;
  kebabName?: string;
  name?: string;
  filePath: string;
  export?: string;
  mode?: "client" | "server";
}

export interface NuxtComponentImport {
  exportName: string;
  filePath: string;
  lazy?: boolean;
  mode?: "client" | "server";
}

export interface NuxtComponentResolverOptions {
  buildDir: string;
  moduleNames?: string[];
  rootDir: string;
}

const COMPONENT_CALL_RE = /_?resolveComponent\s*\(\s*["'`]([^"'`]+)["'`]\s*(?:,\s*[^)]+)?\)/g;
const COMPONENTS_IMPORT_RE = /import\s+(?!type\b)\{([^}]*)\}\s+from\s+(["'])#components\2\s*;?/g;
const COMPONENT_EXT_RE = /\.(?:[cm]?js|ts|vue)$/;
const DTS_COMPONENT_RE =
  /^export const (\w+): (?:LazyComponent<)?typeof import\((["'])(.+?)\2\)(?:\.([A-Za-z_$][\w$]*)|\[['"]([A-Za-z_$][\w$]*)['"]\])>?/;
const DTS_GLOBAL_COMPONENT_RE =
  /^(?:"([^"]+)"|'([^']+)'|([A-Za-z_$][\w$]*))\??:\s*(?:LazyComponent<)?typeof import\((["'])(.+?)\4\)(?:\.([A-Za-z_$][\w$]*)|\[['"]([A-Za-z_$][\w$]*)['"]\])>?;?$/;
const DTS_EXT_RE = /\.d\.ts$/;
const FILE_EXTS = [".js", ".mjs", ".ts", ".vue"];
const CLIENT_COMPONENT_RE = /\.client\.(?:[cm]?js|ts|vue)$/;
const SERVER_COMPONENT_RE = /\.server\.(?:[cm]?js|ts|vue)$/;
const NUXT_ROUTE_ANNOUNCER_RE = /(?:^|[/\\])nuxt-route-announcer\.(?:[cm]?js|ts|vue)$/;
const RUNTIME_COMPONENT_DIRS = [
  "dist/runtime/components",
  "dist/runtime/components/nuxt4",
  "runtime/components",
];
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

function toKebabCase(name: string): string {
  return name
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/_/g, "-")
    .toLowerCase();
}

function toPascalCase(name: string): string {
  return name
    .split(/[-_.]/g)
    .filter(Boolean)
    .map((part) => part[0]!.toUpperCase() + part.slice(1))
    .join("");
}

function addComponentAlias(
  map: Map<string, NuxtComponentImport>,
  name: string | undefined,
  resolved: NuxtComponentImport,
): void {
  if (!name || map.has(name)) {
    return;
  }

  map.set(name, resolved);

  const kebabName = toKebabCase(name);
  if (!map.has(kebabName)) {
    map.set(kebabName, resolved);
  }

  const pascalName = toPascalCase(name);
  if (!map.has(pascalName)) {
    map.set(pascalName, resolved);
  }
}

function addLazyComponentAlias(
  map: Map<string, NuxtComponentImport>,
  name: string | undefined,
  resolved: NuxtComponentImport,
): void {
  if (!name || name.startsWith("Lazy")) {
    return;
  }

  addComponentAlias(map, `Lazy${toPascalCase(name)}`, {
    ...resolved,
    lazy: true,
  });
}

function resolveImportPath(importPath: string): string {
  if (fs.existsSync(importPath)) {
    return importPath;
  }

  for (const ext of FILE_EXTS) {
    const withExt = importPath + ext;
    if (fs.existsSync(withExt)) {
      return withExt;
    }
  }

  return importPath;
}

function isBarePackageSpecifier(importPath: string): boolean {
  if (!importPath) {
    return false;
  }
  if (importPath.startsWith(".")) {
    return false;
  }
  if (importPath.startsWith("/")) {
    return false;
  }
  return !path.isAbsolute(importPath);
}

function resolveDtsImportPath(baseDir: string, importPath: string): string {
  // Nuxt component d.ts entries can name a bare package specifier (e.g. PrimeVue's
  // `Button: typeof import("primevue/button")['default']`). Resolving against the
  // d.ts directory would mint a non-existent `<buildDir>/primevue/button` path that
  // Rollup later fails to resolve. Preserve package specifiers verbatim so the
  // downstream import keeps the original bare specifier.
  if (isBarePackageSpecifier(importPath)) {
    return importPath;
  }
  return resolveImportPath(path.resolve(baseDir, importPath));
}

function detectComponentMode(filePath: string): NuxtComponentImport["mode"] {
  if (CLIENT_COMPONENT_RE.test(filePath)) {
    return "client";
  }
  if (SERVER_COMPONENT_RE.test(filePath)) {
    return "server";
  }
  return undefined;
}

function normalizeComponentMode(mode: unknown): NuxtComponentImport["mode"] {
  return mode === "client" || mode === "server" ? mode : undefined;
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

function createComponentImport(
  filePath: string,
  exportName: string,
  lazy?: boolean,
  mode?: unknown,
): NuxtComponentImport {
  const componentImport: NuxtComponentImport = {
    exportName,
    filePath,
  };

  if (lazy) {
    componentImport.lazy = true;
  }

  const resolvedMode = normalizeComponentMode(mode) ?? detectComponentMode(filePath);
  if (resolvedMode) {
    componentImport.mode = resolvedMode;
  }

  return componentImport;
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

function getNuxtComponentDtsFiles(rootDir: string, buildDir: string): string[] {
  const candidates = [
    path.join(buildDir, "components.d.ts"),
    path.join(buildDir, "types", "components.d.ts"),
    path.join(rootDir, ".nuxt", "components.d.ts"),
    path.join(rootDir, ".nuxt", "types", "components.d.ts"),
    path.join(rootDir, "node_modules", ".cache", "nuxt", ".nuxt", "components.d.ts"),
    path.join(rootDir, "node_modules", ".cache", "nuxt", ".nuxt", "types", "components.d.ts"),
  ];

  return Array.from(new Set(candidates.filter((candidate) => fs.existsSync(candidate))));
}

function forEachLine(content: string, visit: (line: string) => void): void {
  let lineStart = 0;
  for (let index = 0; index <= content.length; index++) {
    if (index !== content.length && content.charCodeAt(index) !== 10) {
      continue;
    }

    const lineEnd = index > lineStart && content.charCodeAt(index - 1) === 13 ? index - 1 : index;
    visit(content.slice(lineStart, lineEnd));
    lineStart = index + 1;
  }
}

function loadDtsComponents(rootDir: string, buildDir: string): Map<string, NuxtComponentImport> {
  const resolved = new Map<string, NuxtComponentImport>();

  for (const filePath of getNuxtComponentDtsFiles(rootDir, buildDir)) {
    let inGlobalComponents = false;
    let braceDepth = 0;

    forEachLine(fs.readFileSync(filePath, "utf-8"), (line) => {
      const trimmed = line.trim();
      if (!inGlobalComponents && trimmed.includes("interface GlobalComponents")) {
        inGlobalComponents = true;
        braceDepth = countBraceDelta(trimmed);
        return;
      }

      if (inGlobalComponents) {
        braceDepth += countBraceDelta(trimmed);
        if (braceDepth <= 0) {
          inGlobalComponents = false;
          return;
        }

        const globalMatch = trimmed.match(DTS_GLOBAL_COMPONENT_RE);
        if (globalMatch) {
          const doubleQuotedName = globalMatch[1];
          const singleQuotedName = globalMatch[2];
          const bareName = globalMatch[3];
          const importPath = globalMatch[5]!;
          const exportNameDot = globalMatch[6];
          const exportNameBracket = globalMatch[7];
          const name = doubleQuotedName || singleQuotedName || bareName;
          const exportName = exportNameDot || exportNameBracket;
          if (name && exportName) {
            const resolvedImportPath = resolveDtsImportPath(path.dirname(filePath), importPath);
            const componentImport = createComponentImport(
              resolvedImportPath,
              exportName,
              name.startsWith("Lazy"),
            );
            addComponentAlias(resolved, name, componentImport);
            addLazyComponentAlias(resolved, name, componentImport);
          }
        }
        return;
      }

      const match = trimmed.match(DTS_COMPONENT_RE);
      if (!match) {
        return;
      }
      const [, name, , importPath, exportNameDot, exportNameBracket] = match;
      const exportName = exportNameDot || exportNameBracket;
      if (!exportName) {
        return;
      }

      const resolvedImportPath = resolveDtsImportPath(path.dirname(filePath), importPath);
      const componentImport = createComponentImport(
        resolvedImportPath,
        exportName,
        name.startsWith("Lazy"),
      );

      addComponentAlias(resolved, name, componentImport);
      addLazyComponentAlias(resolved, name, componentImport);
    });
  }

  return resolved;
}

function countBraceDelta(line: string): number {
  let delta = 0;
  for (const ch of line) {
    if (ch === "{") {
      delta++;
    } else if (ch === "}") {
      delta--;
    }
  }
  return delta;
}

function getProjectPackageNames(moduleNames: string[] | undefined): string[] {
  const packageNames = new Set<string>(["nuxt"]);
  for (const name of moduleNames || []) {
    packageNames.add(name);
  }
  return Array.from(packageNames);
}

function walkRuntimeComponentDir(resolved: Map<string, NuxtComponentImport>, dir: string): void {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkRuntimeComponentDir(resolved, entryPath);
      continue;
    }

    if (!COMPONENT_EXT_RE.test(entry.name) || DTS_EXT_RE.test(entry.name)) {
      continue;
    }

    const baseName = entry.name.replace(COMPONENT_EXT_RE, "");
    const componentName = baseName === "index" ? path.basename(path.dirname(entryPath)) : baseName;
    if (!/[A-Z]/.test(componentName)) {
      continue;
    }

    addComponentAlias(resolved, componentName, {
      ...createComponentImport(entryPath, "default"),
    });
    addLazyComponentAlias(resolved, componentName, {
      ...createComponentImport(entryPath, "default"),
    });
  }
}

function loadRuntimeComponents(
  rootDir: string,
  moduleNames: string[] | undefined,
): Map<string, NuxtComponentImport> {
  const resolved = new Map<string, NuxtComponentImport>();
  const requireFromRoot = createRequire(path.join(rootDir, "package.json"));

  for (const packageName of getProjectPackageNames(moduleNames)) {
    let packageJsonPath = "";
    try {
      packageJsonPath = requireFromRoot.resolve(`${packageName}/package.json`);
    } catch {
      continue;
    }

    const packageDir = path.dirname(packageJsonPath);
    for (const runtimeDir of RUNTIME_COMPONENT_DIRS) {
      const runtimePath = path.join(packageDir, runtimeDir);
      if (fs.existsSync(runtimePath)) {
        walkRuntimeComponentDir(resolved, runtimePath);
      }
    }
  }

  return resolved;
}

export function createNuxtComponentResolver(options: NuxtComponentResolverOptions) {
  const registered = new Map<string, NuxtComponentImport>();
  let dtsResolved: Map<string, NuxtComponentImport> | null = null;
  let runtimeResolved: Map<string, NuxtComponentImport> | null = null;

  function getDtsResolved(): Map<string, NuxtComponentImport> {
    if (!dtsResolved) {
      dtsResolved = loadDtsComponents(options.rootDir, options.buildDir);
    }
    return dtsResolved;
  }

  function getRuntimeResolved(): Map<string, NuxtComponentImport> {
    if (!runtimeResolved) {
      runtimeResolved = loadRuntimeComponents(options.rootDir, options.moduleNames);
    }
    return runtimeResolved;
  }

  return {
    register(components: NuxtComponentDescriptor[]): void {
      for (const component of components) {
        const resolved = createComponentImport(
          component.filePath,
          component.export || "default",
          false,
          component.mode,
        );
        addComponentAlias(registered, component.pascalName, resolved);
        addComponentAlias(registered, component.kebabName, resolved);
        addComponentAlias(registered, component.name, resolved);
        addLazyComponentAlias(registered, component.pascalName, resolved);
        addLazyComponentAlias(registered, component.kebabName, resolved);
        addLazyComponentAlias(registered, component.name, resolved);
      }
    },

    resolve(name: string): NuxtComponentImport | null {
      const normalizedName = name.trim();
      const directResolved = registered.get(normalizedName) ?? getDtsResolved().get(normalizedName);
      if (directResolved) {
        return directResolved;
      }

      if (!/[A-Z]/.test(normalizedName)) {
        return null;
      }

      return getRuntimeResolved().get(normalizedName) ?? null;
    },
  };
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
          componentImports,
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

      if (unresolvedSpecifiers.length === 0) {
        return "";
      }

      return `import { ${unresolvedSpecifiers.join(", ")} } from "#components";`;
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

  if (componentImports.length === 0) {
    return code;
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
