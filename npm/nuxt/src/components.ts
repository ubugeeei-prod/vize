import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

export interface NuxtComponentDescriptor {
  pascalName?: string;
  kebabName?: string;
  name?: string;
  filePath: string;
  export?: string;
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
const COMPONENT_EXT_RE = /\.(?:[cm]?js|ts|vue)$/;
const DTS_COMPONENT_RE =
  /^export const (\w+): (?:LazyComponent<)?typeof import\((["'])(.+?)\2\)(?:\.([A-Za-z_$][\w$]*)|\[['"]([A-Za-z_$][\w$]*)['"]\])>?/;
const DTS_EXT_RE = /\.d\.ts$/;
const FILE_EXTS = [".js", ".mjs", ".ts", ".vue"];
const CLIENT_COMPONENT_RE = /\.client\.(?:[cm]?js|ts|vue)$/;
const SERVER_COMPONENT_RE = /\.server\.(?:[cm]?js|ts|vue)$/;
const RUNTIME_COMPONENT_DIRS = [
  "dist/runtime/components",
  "dist/runtime/components/nuxt4",
  "runtime/components",
];

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

function detectComponentMode(filePath: string): NuxtComponentImport["mode"] {
  if (CLIENT_COMPONENT_RE.test(filePath)) {
    return "client";
  }
  if (SERVER_COMPONENT_RE.test(filePath)) {
    return "server";
  }
  return undefined;
}

function createComponentImport(
  filePath: string,
  exportName: string,
  lazy?: boolean,
): NuxtComponentImport {
  const componentImport: NuxtComponentImport = {
    exportName,
    filePath,
  };

  if (lazy) {
    componentImport.lazy = true;
  }

  const mode = detectComponentMode(filePath);
  if (mode) {
    componentImport.mode = mode;
  }

  return componentImport;
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

function loadDtsComponents(rootDir: string, buildDir: string): Map<string, NuxtComponentImport> {
  const resolved = new Map<string, NuxtComponentImport>();

  for (const filePath of getNuxtComponentDtsFiles(rootDir, buildDir)) {
    const lines = fs.readFileSync(filePath, "utf-8").split("\n");
    for (const line of lines) {
      const match = line.match(DTS_COMPONENT_RE);
      if (!match) {
        continue;
      }

      const [, name, , importPath, exportNameDot, exportNameBracket] = match;
      const exportName = exportNameDot || exportNameBracket;
      if (!exportName) {
        continue;
      }

      const absoluteImportPath = resolveImportPath(
        path.resolve(path.dirname(filePath), importPath),
      );
      const componentImport = createComponentImport(
        absoluteImportPath,
        exportName,
        name.startsWith("Lazy"),
      );

      addComponentAlias(resolved, name, componentImport);
      addLazyComponentAlias(resolved, name, componentImport);
    }
  }

  return resolved;
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
        const resolved = createComponentImport(component.filePath, component.export || "default");
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
  let needsDefineAsyncComponent = false;
  let needsCreateClientOnly = false;

  const nextCode = code.replace(COMPONENT_CALL_RE, (match: string, name: string) => {
    const resolved = resolveComponentImport(name);
    if (!resolved) {
      return match;
    }

    const importKey = `${resolved.exportName}\u0000${resolved.filePath}\u0000${resolved.lazy ? "lazy" : "eager"}\u0000${resolved.mode ?? "default"}`;
    let variableName = importedComponents.get(importKey);
    if (!variableName) {
      variableName = `__nuxt_component_${counter++}`;
      importedComponents.set(importKey, variableName);
      if (resolved.lazy) {
        needsDefineAsyncComponent = true;
        const exportAccessor =
          resolved.exportName === "default"
            ? "module.default"
            : `module[${JSON.stringify(resolved.exportName)}]`;
        if (resolved.mode === "client") {
          needsCreateClientOnly = true;
          componentImports.push(
            `const ${variableName} = __nuxt_define_async_component(() => import(${JSON.stringify(resolved.filePath)}).then((module) => __nuxt_create_client_only(${exportAccessor})));`,
          );
        } else {
          componentImports.push(
            `const ${variableName} = __nuxt_define_async_component(() => import(${JSON.stringify(resolved.filePath)}).then((module) => ${exportAccessor}));`,
          );
        }
      } else if (resolved.exportName === "default") {
        if (resolved.mode === "client") {
          needsCreateClientOnly = true;
          const rawVariableName = `${variableName}_raw`;
          componentImports.push(
            `import ${rawVariableName} from ${JSON.stringify(resolved.filePath)};`,
          );
          componentImports.push(
            `const ${variableName} = __nuxt_create_client_only(${rawVariableName});`,
          );
        } else {
          componentImports.push(
            `import ${variableName} from ${JSON.stringify(resolved.filePath)};`,
          );
        }
      } else {
        if (resolved.mode === "client") {
          needsCreateClientOnly = true;
          const rawVariableName = `${variableName}_raw`;
          componentImports.push(
            `import { ${resolved.exportName} as ${rawVariableName} } from ${JSON.stringify(resolved.filePath)};`,
          );
          componentImports.push(
            `const ${variableName} = __nuxt_create_client_only(${rawVariableName});`,
          );
        } else {
          componentImports.push(
            `import { ${resolved.exportName} as ${variableName} } from ${JSON.stringify(resolved.filePath)};`,
          );
        }
      }
    }

    return variableName;
  });

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
