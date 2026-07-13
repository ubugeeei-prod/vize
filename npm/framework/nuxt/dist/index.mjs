import { createRequire } from "node:module";
import fs from "node:fs";
import path, { dirname, resolve } from "node:path";
import { parseSync } from "vite";
import { createHash } from "node:crypto";
//#region src/components.ts
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
function toKebabCase(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/_/g, "-")
    .toLowerCase();
}
function toPascalCase(name) {
  return name
    .split(/[-_.]/g)
    .filter(Boolean)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
}
function addComponentAlias(map, name, resolved) {
  if (!name || map.has(name)) return;
  map.set(name, resolved);
  const kebabName = toKebabCase(name);
  if (!map.has(kebabName)) map.set(kebabName, resolved);
  const pascalName = toPascalCase(name);
  if (!map.has(pascalName)) map.set(pascalName, resolved);
}
function addLazyComponentAlias(map, name, resolved) {
  if (!name || name.startsWith("Lazy")) return;
  addComponentAlias(map, `Lazy${toPascalCase(name)}`, {
    ...resolved,
    lazy: true,
  });
}
function resolveImportPath(importPath) {
  if (fs.existsSync(importPath)) return importPath;
  for (const ext of FILE_EXTS) {
    const withExt = importPath + ext;
    if (fs.existsSync(withExt)) return withExt;
  }
  return importPath;
}
function isBarePackageSpecifier(importPath) {
  if (!importPath) return false;
  if (importPath.startsWith(".")) return false;
  if (importPath.startsWith("/")) return false;
  return !path.isAbsolute(importPath);
}
function resolveDtsImportPath(baseDir, importPath) {
  if (isBarePackageSpecifier(importPath)) return importPath;
  return resolveImportPath(path.resolve(baseDir, importPath));
}
function detectComponentMode(filePath) {
  if (CLIENT_COMPONENT_RE.test(filePath)) return "client";
  if (SERVER_COMPONENT_RE.test(filePath)) return "server";
}
function normalizeComponentMode(mode) {
  return mode === "client" || mode === "server" ? mode : void 0;
}
function needsClientOnlyWrapper(resolved) {
  if (resolved.mode !== "client") return false;
  return !NUXT_ROUTE_ANNOUNCER_RE.test(resolved.filePath);
}
function parseComponentImportSpecifier(raw) {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  const match = trimmed.match(IMPORT_SPECIFIER_RE);
  if (!match) return null;
  const [, typeKeyword, importedName, localName] = match;
  return {
    importedName,
    localName: localName || importedName,
    typeOnly: Boolean(typeKeyword),
  };
}
function splitComponentImportSpecifiers(specifiers) {
  return specifiers
    .split(",")
    .map((specifier) => specifier.trim())
    .filter(Boolean);
}
function createComponentImport(filePath, exportName, lazy, mode) {
  const componentImport = {
    exportName,
    filePath,
  };
  if (lazy) componentImport.lazy = true;
  const resolvedMode = normalizeComponentMode(mode) ?? detectComponentMode(filePath);
  if (resolvedMode) componentImport.mode = resolvedMode;
  return componentImport;
}
function addResolvedComponentBinding(componentImports, resolved, variableName, rawVariableName) {
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
    } else
      componentImports.push(
        `const ${variableName} = __nuxt_define_async_component(() => import(${JSON.stringify(resolved.filePath)}).then((module) => ${exportAccessor}));`,
      );
    return {
      needsCreateClientOnly,
      needsDefineAsyncComponent,
    };
  }
  if (resolved.exportName === "default") {
    if (wrapClientOnly) {
      needsCreateClientOnly = true;
      componentImports.push(`import ${rawVariableName} from ${JSON.stringify(resolved.filePath)};`);
      componentImports.push(
        `const ${variableName} = __nuxt_create_client_only(${rawVariableName});`,
      );
    } else
      componentImports.push(`import ${variableName} from ${JSON.stringify(resolved.filePath)};`);
    return {
      needsCreateClientOnly,
      needsDefineAsyncComponent,
    };
  }
  if (wrapClientOnly) {
    needsCreateClientOnly = true;
    componentImports.push(
      `import { ${resolved.exportName} as ${rawVariableName} } from ${JSON.stringify(resolved.filePath)};`,
    );
    componentImports.push(`const ${variableName} = __nuxt_create_client_only(${rawVariableName});`);
  } else
    componentImports.push(
      `import { ${resolved.exportName} as ${variableName} } from ${JSON.stringify(resolved.filePath)};`,
    );
  return {
    needsCreateClientOnly,
    needsDefineAsyncComponent,
  };
}
function getNuxtComponentDtsFiles(rootDir, buildDir) {
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
function forEachLine(content, visit) {
  let lineStart = 0;
  for (let index = 0; index <= content.length; index++) {
    if (index !== content.length && content.charCodeAt(index) !== 10) continue;
    const lineEnd = index > lineStart && content.charCodeAt(index - 1) === 13 ? index - 1 : index;
    visit(content.slice(lineStart, lineEnd));
    lineStart = index + 1;
  }
}
function loadDtsComponents(rootDir, buildDir) {
  const resolved = /* @__PURE__ */ new Map();
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
          const importPath = globalMatch[5];
          const exportNameDot = globalMatch[6];
          const exportNameBracket = globalMatch[7];
          const name = doubleQuotedName || singleQuotedName || bareName;
          const exportName = exportNameDot || exportNameBracket;
          if (name && exportName) {
            const componentImport = createComponentImport(
              resolveDtsImportPath(path.dirname(filePath), importPath),
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
      if (!match) return;
      const [, name, , importPath, exportNameDot, exportNameBracket] = match;
      const exportName = exportNameDot || exportNameBracket;
      if (!exportName) return;
      const componentImport = createComponentImport(
        resolveDtsImportPath(path.dirname(filePath), importPath),
        exportName,
        name.startsWith("Lazy"),
      );
      addComponentAlias(resolved, name, componentImport);
      addLazyComponentAlias(resolved, name, componentImport);
    });
  }
  return resolved;
}
function countBraceDelta(line) {
  let delta = 0;
  for (const ch of line)
    if (ch === "{") delta++;
    else if (ch === "}") delta--;
  return delta;
}
function getProjectPackageNames(moduleNames) {
  const packageNames = new Set(["nuxt"]);
  for (const name of moduleNames || []) packageNames.add(name);
  return Array.from(packageNames);
}
function walkRuntimeComponentDir(resolved, dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkRuntimeComponentDir(resolved, entryPath);
      continue;
    }
    if (!COMPONENT_EXT_RE.test(entry.name) || DTS_EXT_RE.test(entry.name)) continue;
    const baseName = entry.name.replace(COMPONENT_EXT_RE, "");
    const componentName = baseName === "index" ? path.basename(path.dirname(entryPath)) : baseName;
    if (!/[A-Z]/.test(componentName)) continue;
    addComponentAlias(resolved, componentName, { ...createComponentImport(entryPath, "default") });
    addLazyComponentAlias(resolved, componentName, {
      ...createComponentImport(entryPath, "default"),
    });
  }
}
function loadRuntimeComponents(rootDir, moduleNames) {
  const resolved = /* @__PURE__ */ new Map();
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
      if (fs.existsSync(runtimePath)) walkRuntimeComponentDir(resolved, runtimePath);
    }
  }
  return resolved;
}
function createNuxtComponentResolver(options) {
  const registered = /* @__PURE__ */ new Map();
  let dtsResolved = null;
  let runtimeResolved = null;
  function getDtsResolved() {
    if (!dtsResolved) dtsResolved = loadDtsComponents(options.rootDir, options.buildDir);
    return dtsResolved;
  }
  function getRuntimeResolved() {
    if (!runtimeResolved)
      runtimeResolved = loadRuntimeComponents(options.rootDir, options.moduleNames);
    return runtimeResolved;
  }
  return {
    register(components) {
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
    resolve(name) {
      const normalizedName = name.trim();
      const directResolved = registered.get(normalizedName) ?? getDtsResolved().get(normalizedName);
      if (directResolved) return directResolved;
      if (!/[A-Z]/.test(normalizedName)) return null;
      return getRuntimeResolved().get(normalizedName) ?? null;
    },
  };
}
function injectNuxtComponentImports(code, resolveComponentImport) {
  const componentImports = [];
  const importedComponents = /* @__PURE__ */ new Map();
  let counter = 0;
  let importCounter = 0;
  let needsDefineAsyncComponent = false;
  let needsCreateClientOnly = false;
  const nextCode = code
    .replace(COMPONENTS_IMPORT_RE, (match, specifiers) => {
      const unresolvedSpecifiers = [];
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
      if (!changed) return match;
      if (unresolvedSpecifiers.length === 0) return "";
      return `import { ${unresolvedSpecifiers.join(", ")} } from "#components";`;
    })
    .replace(COMPONENT_CALL_RE, (match, name) => {
      const resolved = resolveComponentImport(name);
      if (!resolved) return match;
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
    });
  if (componentImports.length === 0) return code;
  return (
    [
      ...(needsDefineAsyncComponent
        ? ['import { defineAsyncComponent as __nuxt_define_async_component } from "vue";']
        : []),
      ...(needsCreateClientOnly
        ? [
            'import { createClientOnly as __nuxt_create_client_only } from "#app/components/client-only";',
          ]
        : []),
      ...componentImports,
    ].join("\n") +
    "\n" +
    nextCode
  );
}
//#endregion
//#region src/i18n.ts
const I18N_FN_MAP = {
  $t: "t: $t",
  $rt: "rt: $rt",
  $d: "d: $d",
  $n: "n: $n",
  $tm: "tm: $tm",
  $te: "te: $te",
};
const I18N_HELPER_LOCALS = new Set(Object.keys(I18N_FN_MAP));
const DEFAULT_PARSE_ID = "vize-nuxt-i18n-bridge.tsx";
function getLocalAlias(specifier) {
  const colon = specifier.indexOf(":");
  return (colon === -1 ? specifier : specifier.slice(colon + 1)).trim();
}
function isNode(value) {
  return value != null && typeof value === "object" && typeof value.type === "string";
}
function getNodeStart(node) {
  return typeof node.start === "number" ? node.start : null;
}
function getNodeEnd(node) {
  return typeof node.end === "number" ? node.end : null;
}
function getNodeName(node) {
  return isNode(node) && typeof node.name === "string" ? node.name : null;
}
function getChildNodes(node) {
  const children = [];
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) if (isNode(item)) children.push(item);
      continue;
    }
    if (isNode(value)) children.push(value);
  }
  return children;
}
function walkNode(node, visit) {
  visit(node);
  for (const child of getChildNodes(node)) walkNode(child, visit);
}
function parseProgram(code, id) {
  try {
    const result = parseSync(normalizeParseId(id), code);
    if (isNode(result)) return result;
    if (result != null && typeof result === "object") {
      const program = result.program;
      if (isNode(program)) return program;
    }
    return null;
  } catch {
    return null;
  }
}
function normalizeParseId(id) {
  return (id.startsWith("\0") ? id.slice(1) : id).split(/[?#]/, 1)[0] || DEFAULT_PARSE_ID;
}
function isSetupFunction(node) {
  if (node.type !== "Property") return false;
  const key = isNode(node.key) ? node.key : null;
  const value = isNode(node.value) ? node.value : null;
  if (getNodeName(key) !== "setup" || !value) return false;
  if (value.type !== "FunctionExpression" && value.type !== "ArrowFunctionExpression") return false;
  return getNodeName((Array.isArray(value.params) ? value.params : [])[0]) === "__props";
}
function findSetupFunctionBody(program) {
  let setupBody = null;
  walkNode(program, (node) => {
    if (setupBody || !isSetupFunction(node)) return;
    const value = isNode(node.value) ? node.value : null;
    const body = value && isNode(value.body) ? value.body : null;
    if (body?.type === "BlockStatement") setupBody = body;
  });
  return setupBody;
}
function isUseI18nCall(node) {
  if (!isNode(node) || node.type !== "CallExpression") return false;
  return getNodeName(isNode(node.callee) ? node.callee : null) === "useI18n";
}
function collectPatternBindingNames(pattern, locals) {
  if (pattern.type === "Identifier" && typeof pattern.name === "string") {
    locals.add(pattern.name);
    return;
  }
  if (pattern.type === "AssignmentPattern" && isNode(pattern.left)) {
    collectPatternBindingNames(pattern.left, locals);
    return;
  }
  if (pattern.type === "RestElement" && isNode(pattern.argument)) {
    collectPatternBindingNames(pattern.argument, locals);
    return;
  }
  if (pattern.type === "Property" && isNode(pattern.value)) {
    collectPatternBindingNames(pattern.value, locals);
    return;
  }
  for (const child of getChildNodes(pattern)) collectPatternBindingNames(child, locals);
}
function findExistingUseI18nDestructure(setupBody) {
  const statements = Array.isArray(setupBody.body) ? setupBody.body : [];
  for (const statement of statements) {
    if (!isNode(statement) || statement.type !== "VariableDeclaration") continue;
    const declarations = Array.isArray(statement.declarations) ? statement.declarations : [];
    for (const declaration of declarations) {
      if (
        !isNode(declaration) ||
        !isNode(declaration.id) ||
        declaration.id.type !== "ObjectPattern"
      )
        continue;
      if (!isUseI18nCall(isNode(declaration.init) ? declaration.init : null)) continue;
      const locals = /* @__PURE__ */ new Set();
      collectPatternBindingNames(declaration.id, locals);
      return {
        declaration: statement,
        pattern: declaration.id,
        locals,
      };
    }
  }
  return null;
}
function collectUsedI18nSpecifiers(setupBody) {
  const used = /* @__PURE__ */ new Map();
  walkNode(setupBody, (node) => {
    if (node.type !== "CallExpression") return;
    const fnName = getNodeName(isNode(node.callee) ? node.callee : null);
    const callStart = getNodeStart(node);
    if (!fnName || callStart == null || !I18N_HELPER_LOCALS.has(fnName)) return;
    if (!used.has(fnName)) used.set(fnName, callStart);
  });
  const helpers = Array.from(used.entries()).sort((a, b) => a[1] - b[1]);
  return {
    firstUseIndex: helpers[0]?.[1] ?? Number.POSITIVE_INFINITY,
    specifiers: helpers.map(([name]) => I18N_FN_MAP[name]).filter((value) => value != null),
  };
}
function injectNuxtI18nHelpers(code, id = DEFAULT_PARSE_ID) {
  const program = parseProgram(code, id);
  if (!program) return code;
  const setupBody = findSetupFunctionBody(program);
  const setupBodyStart = setupBody ? getNodeStart(setupBody) : null;
  if (!setupBody || setupBodyStart == null) return code;
  const insertAt = setupBodyStart + 1;
  const { firstUseIndex, specifiers: usedSpecifiers } = collectUsedI18nSpecifiers(setupBody);
  if (usedSpecifiers.length === 0) return code;
  const existing = findExistingUseI18nDestructure(setupBody);
  if (existing) {
    const missingSpecifiers = usedSpecifiers.filter((specifier) => {
      return !existing.locals.has(getLocalAlias(specifier));
    });
    if (missingSpecifiers.length === 0) return code;
    const declarationStart = getNodeStart(existing.declaration);
    const declarationEnd = getNodeEnd(existing.declaration);
    const patternStart = getNodeStart(existing.pattern);
    const patternEnd = getNodeEnd(existing.pattern);
    if (
      declarationStart == null ||
      declarationEnd == null ||
      patternStart == null ||
      patternEnd == null
    )
      return code;
    if (declarationStart > firstUseIndex)
      return (
        code.slice(0, insertAt) +
        `\nconst { ${missingSpecifiers.join(", ")} } = useI18n();\n` +
        code.slice(insertAt)
      );
    const merged = code.slice(patternStart + 1, patternEnd - 1).trim();
    const nextDestructure = merged
      ? `${merged}, ${missingSpecifiers.join(", ")}`
      : missingSpecifiers.join(", ");
    return (
      code.slice(0, declarationStart) +
      `const { ${nextDestructure} } = useI18n();` +
      code.slice(declarationEnd)
    );
  }
  return (
    code.slice(0, insertAt) +
    `\nconst { ${usedSpecifiers.join(", ")} } = useI18n();\n` +
    code.slice(insertAt)
  );
}
//#endregion
//#region src/musea-components.ts
const MUSEA_ART_COMPONENT_IGNORE = "**/*.art.vue";
function appendMuseaArtComponentIgnore(dirs) {
  for (const [index, dir] of dirs.entries()) {
    if (typeof dir === "string") {
      dirs[index] = {
        path: dir,
        ignore: [MUSEA_ART_COMPONENT_IGNORE],
      };
      continue;
    }
    const ignore = Array.isArray(dir.ignore) ? dir.ignore : [];
    if (ignore.includes("**/*.art.vue")) continue;
    dir.ignore = [...ignore, MUSEA_ART_COMPONENT_IGNORE];
  }
}
//#endregion
//#region src/musea-static.ts
function registerNuxtMuseaStaticPublicAsset(nuxt, basePath) {
  nuxt.hook("nitro:config", (nitroConfig) => {
    nitroConfig.publicAssets = [
      ...(nitroConfig.publicAssets ?? []),
      resolveNuxtMuseaStaticPublicAsset(nuxt.options.rootDir, nuxt.options.buildDir, basePath),
    ];
  });
}
function resolveNuxtMuseaStaticPublicAsset(rootDir, buildDir, basePath) {
  const staticRoot = museaStaticRootFromBasePath(basePath);
  const resolvedBuildDir = path.isAbsolute(buildDir) ? buildDir : path.resolve(rootDir, buildDir);
  return {
    dir: path.join(resolvedBuildDir, "dist", "client", staticRoot),
    baseURL: normalizeMuseaBasePath(basePath),
  };
}
function museaStaticRootFromBasePath(basePath) {
  return basePath.replace(/^\/+|\/+$/g, "");
}
function normalizeMuseaBasePath(basePath) {
  const normalized = basePath.replace(/^\/+|\/+$/g, "");
  return normalized ? `/${normalized}` : "/";
}
//#endregion
//#region src/bridge-fast-path.ts
const VUE_CLIENT_RUNTIME_IMPORT = "vue/dist/vue.runtime.esm-bundler.js";
const COMPONENT_BRIDGE_RE = /(?:_?resolveComponent\s*\(|from\s+(["'])#components\1)/;
const I18N_BRIDGE_RE = /\b(?:\$t|\$rt|\$d|\$n|\$tm|\$te)\s*\(/;
const STABLE_KEY_BRIDGE_RE = /\b(?:useFetch|useLazyFetch)\s*\(|\/\*\s*nuxt-injected\s*\*\//;
function hasComponentBridgeInput(code) {
  return COMPONENT_BRIDGE_RE.test(code);
}
function hasI18nBridgeInput(code) {
  return I18N_BRIDGE_RE.test(code);
}
function hasStableKeyBridgeInput(code) {
  return STABLE_KEY_BRIDGE_RE.test(code);
}
function rewriteBareVueImportsToClientRuntime(code) {
  return code
    .replace(/(\bfrom\s*)(["'])vue\2/g, (_, prefix, quote) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    })
    .replace(/(\bimport\s*)(["'])vue\2/g, (_, prefix, quote) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    });
}
//#endregion
//#region src/utils.ts
const NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE = /\.takumi\.vue(?:\?|$)/;
function normalizeUrlPrefix(value) {
  const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
  return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
}
function buildNuxtDevAssetBase(baseURL = "/", buildAssetsDir = "/_nuxt/") {
  const normalizedBase = normalizeUrlPrefix(baseURL);
  const normalizedAssetsDir = normalizeUrlPrefix(buildAssetsDir);
  return normalizedBase === "/"
    ? normalizedAssetsDir
    : normalizeUrlPrefix(`${normalizedBase}${normalizedAssetsDir.replace(/^\//, "")}`);
}
function buildNuxtCompilerOptions(
  rootDir,
  baseURL = "/",
  buildAssetsDir = "/_nuxt/",
  overrides = {},
) {
  const defaults = {
    devUrlBase: buildNuxtDevAssetBase(baseURL, buildAssetsDir),
    handleNodeModulesVue: false,
    root: rootDir,
    scanPatterns: [],
  };
  if (overrides.customRenderer === true && overrides.exclude !== void 0)
    defaults.exclude = overrides.exclude;
  else if (overrides.customRenderer !== true)
    defaults.exclude = mergeNuxtCompilerPatterns(
      NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      overrides.exclude,
    );
  for (const [key, value] of Object.entries(overrides)) {
    if (key === "exclude") continue;
    if (value !== void 0) defaults[key] = value;
  }
  return defaults;
}
function mergeNuxtCompilerPatterns(defaultPattern, userPattern) {
  if (userPattern == null) return defaultPattern;
  return [defaultPattern, ...(Array.isArray(userPattern) ? userPattern : [userPattern])];
}
function isVizeVirtualVueModuleId(id) {
  return id.startsWith("\0") && /\.vue\.tsx?(?:\?|$)/.test(id);
}
function isVizeGeneratedVueModuleId(id) {
  let normalized = id;
  if (normalized.startsWith("/@id/__x00__")) normalized = normalized.slice(12);
  else if (normalized.startsWith("__x00__")) normalized = normalized.slice(7);
  return /\.vue\.tsx?(?:\?|$)/.test(normalized);
}
/**
 * Recognize raw `.jsx`/`.tsx` Vue component modules compiled by Vize.
 *
 * Unlike `.vue` files, JSX/TSX modules are transformed in place by the
 * underlying Vite plugin (the original `.jsx`/`.tsx` id is preserved, no
 * `\0`-prefixed `.vue.ts[x]` virtual id is created). Nuxt's auto-import,
 * component, and i18n transforms still need to run on these modules, so the
 * Nuxt transform bridge keys off this predicate in addition to
 * `isVizeGeneratedVueModuleId`.
 *
 * A bare query suffix (e.g. `?vue`) is ignored so dev-server requests still
 * match, but `?raw`/`?url`/`?worker` asset imports are rejected since those
 * are not compiled component modules.
 */
function isVizeJsxModuleId(id) {
  const queryIndex = id.indexOf("?");
  const pathPart = queryIndex === -1 ? id : id.slice(0, queryIndex);
  if (/\.vue\.tsx?$/.test(pathPart) || !/\.(?:jsx|tsx)$/.test(pathPart)) return false;
  if (queryIndex === -1) return true;
  const params = new URLSearchParams(id.slice(queryIndex + 1));
  return !(
    params.has("raw") ||
    params.has("url") ||
    params.has("worker") ||
    params.has("sharedworker")
  );
}
function normalizeVizeVirtualVueModuleId(id) {
  return (id.startsWith("\0vize-ssr:") ? id.slice(10) : id.slice(1)).replace(/\.tsx?(?=\?|$)/, "");
}
function normalizeVizeGeneratedVueModuleId(id) {
  if (isVizeVirtualVueModuleId(id)) return normalizeVizeVirtualVueModuleId(id);
  return id
    .replace(/^\/@id\/__x00__/, "")
    .replace(/^__x00__/, "")
    .replace(/\.tsx?(?=\?|$)/, "");
}
const NUXT_INJECTED_MARKER = "/* nuxt-injected */";
const NUXT_INJECTED_KEY_RE = /'\$[^']+'\s+\/\* nuxt-injected \*\//g;
const NUXT_FETCH_COMPOSABLE_RE = /\b(?:useFetch|useLazyFetch)\s*\(/g;
function buildStableNuxtKey(id, index) {
  return createHash("sha256")
    .update(id)
    .update(":")
    .update(String(index))
    .digest("base64url")
    .slice(0, 10);
}
function normalizeNuxtInjectedKeysForVizeVirtualModule$1(code, id) {
  const normalizedId = normalizeVizeGeneratedVueModuleId(id).replace(/\?.*$/, "");
  let index = 0;
  return code.replace(NUXT_INJECTED_KEY_RE, () => {
    index += 1;
    return `'$${buildStableNuxtKey(normalizedId, index)}' ${NUXT_INJECTED_MARKER}`;
  });
}
function stabilizeNuxtInjectedKeysForVizeVirtualModule(code, id) {
  return normalizeNuxtInjectedKeysForVizeVirtualModule$1(injectMissingNuxtFetchKeys(code), id);
}
function injectMissingNuxtFetchKeys(code) {
  let output = "";
  let cursor = 0;
  for (const match of code.matchAll(NUXT_FETCH_COMPOSABLE_RE)) {
    const openParenIndex = (match.index ?? 0) + match[0].length - 1;
    if (openParenIndex < cursor) continue;
    const closeParenIndex = findMatchingParen(code, openParenIndex);
    if (closeParenIndex === -1) continue;
    const args = code.slice(openParenIndex + 1, closeParenIndex);
    if (args.includes(NUXT_INJECTED_MARKER)) continue;
    output += code.slice(cursor, closeParenIndex);
    output += `${args.trim().length === 0 ? "" : ", "}'$__vize_nuxt_key__' ${NUXT_INJECTED_MARKER}`;
    cursor = closeParenIndex;
  }
  return cursor === 0 ? code : output + code.slice(cursor);
}
function findMatchingParen(code, openParenIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = openParenIndex; index < code.length; index += 1) {
    const char = code[index];
    const next = code[index + 1];
    if (lineComment) {
      if (char === "\n" || char === "\r") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (char === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (char === "\\") escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (char === "'" || char === '"' || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(") {
      depth += 1;
      continue;
    }
    if (char === ")") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}
const NAMED_IMPORT_RE = /^import\s*\{([\s\S]*?)\}\s*from\s*(['"])(vue|#imports|#entry)\2\s*;?/gm;
function parseNamedImportSpecifiers(specifierSource) {
  return specifierSource
    .split(",")
    .map((specifier) => specifier.trim())
    .filter(Boolean)
    .flatMap((specifier) => {
      const withoutType = specifier.replace(/^type\s+/, "").trim();
      if (!withoutType) return [];
      const match = withoutType.match(/^([A-Za-z_$][\w$]*)(?:\s+as\s+([A-Za-z_$][\w$]*))?$/);
      if (!match) return [];
      const imported = match[1];
      return [
        {
          imported,
          local: match[2] ?? imported,
          raw: withoutType,
        },
      ];
    });
}
function collectNamedImports(code) {
  const imports = [];
  for (const match of code.matchAll(NAMED_IMPORT_RE))
    imports.push({
      start: match.index ?? 0,
      end: (match.index ?? 0) + match[0].length,
      quote: match[2],
      source: match[3],
      specifiers: parseNamedImportSpecifiers(match[1] ?? ""),
    });
  return imports;
}
function renderNamedImport(specifiers, source, quote) {
  return `import { ${specifiers.map((specifier) => specifier.raw).join(", ")} } from ${quote}${source}${quote};`;
}
function preserveExplicitVueImportsFromNuxtAutoImports(originalCode, injectedCode) {
  const originalVueSpecifiers = /* @__PURE__ */ new Map();
  for (const statement of collectNamedImports(originalCode)) {
    if (statement.source !== "vue") continue;
    for (const specifier of statement.specifiers)
      originalVueSpecifiers.set(specifier.local, specifier);
  }
  if (originalVueSpecifiers.size === 0) return injectedCode;
  const restoredSpecifiers = /* @__PURE__ */ new Map();
  const replacements = [];
  for (const statement of collectNamedImports(injectedCode)) {
    if (statement.source !== "#imports" && statement.source !== "#entry") continue;
    const keep = [];
    let changed = false;
    for (const specifier of statement.specifiers) {
      const original = originalVueSpecifiers.get(specifier.local);
      if (original) {
        restoredSpecifiers.set(specifier.local, original);
        changed = true;
      } else keep.push(specifier);
    }
    if (!changed) continue;
    replacements.push({
      start: statement.start,
      end: statement.end,
      text: keep.length > 0 ? renderNamedImport(keep, statement.source, statement.quote) : "",
    });
  }
  if (replacements.length === 0) return injectedCode;
  let output = injectedCode;
  for (const replacement of replacements.reverse())
    output = output.slice(0, replacement.start) + replacement.text + output.slice(replacement.end);
  const currentVueLocals = /* @__PURE__ */ new Set();
  for (const statement of collectNamedImports(output)) {
    if (statement.source !== "vue") continue;
    for (const specifier of statement.specifiers) currentVueLocals.add(specifier.local);
  }
  const missing = [...restoredSpecifiers.values()].filter(
    (specifier) => !currentVueLocals.has(specifier.local),
  );
  if (missing.length > 0) output = `${renderNamedImport(missing, "vue", '"')}\n${output}`;
  return output.replace(/\n{3,}/g, "\n\n");
}
function preserveExplicitVueImportsFromVizeModuleSource(id, code) {
  if (!isVizeVirtualVueModuleId(id) && !isVizeGeneratedVueModuleId(id)) return code;
  const sourcePath = normalizeVizeGeneratedVueModuleId(id).replace(/\?.*$/, "");
  if (!sourcePath.endsWith(".vue") || !fs.existsSync(sourcePath)) return code;
  return preserveExplicitVueImportsFromNuxtAutoImports(fs.readFileSync(sourcePath, "utf-8"), code);
}
//#endregion
//#region src/options.ts
const DEFAULT_NUXT_BRIDGE_OPTIONS = {
  autoImports: true,
  components: true,
  i18n: true,
  stableInjectedKeys: true,
};
const DEFAULT_NUXT_UNOCSS_OPTIONS = { originalSource: {} };
const DEFAULT_NUXT_DEV_OPTIONS = { stylesheetLinks: true };
function isLegacyVueVersion(version) {
  return (
    version === 0.11 || version === 1 || version === 2 || version === "2.7" || version === "legacy"
  );
}
function normalizeNuxtCompilerCompatibilityOptions(compatibility) {
  const normalized = {};
  const legacyHost =
    isLegacyVueVersion(compatibility.vueVersion) || compatibility.nuxtVersion === 2;
  if (compatibility.vueVersion !== void 0) normalized.vueVersion = compatibility.vueVersion;
  if (compatibility.hostCompiler !== void 0 || legacyHost)
    normalized.hostCompiler = compatibility.hostCompiler ?? true;
  if (compatibility.scriptSetupInStandalone !== void 0)
    normalized.scriptSetupInStandalone = compatibility.scriptSetupInStandalone;
  if (compatibility.optionsApiVapor !== void 0)
    normalized.optionsApiVapor = compatibility.optionsApiVapor;
  if (compatibility.nuxtVersion !== void 0) normalized.nuxtVersion = compatibility.nuxtVersion;
  if (compatibility.webpackVersion !== void 0)
    normalized.webpackVersion = compatibility.webpackVersion;
  return normalized;
}
function resolveNuxtCompilerOptions(
  rootDir,
  baseURL,
  buildAssetsDir,
  compiler,
  compatibility = {},
) {
  if (compiler === false) return false;
  if (compatibility.supportsViteCompiler === false && compatibility.forceViteCompiler !== true)
    return false;
  const compatibilityOptions = normalizeNuxtCompilerCompatibilityOptions(compatibility);
  const hasCompatibilityOptions = Object.keys(compatibilityOptions).length > 0;
  const overrides = typeof compiler === "object" && compiler != null ? compiler : {};
  return buildNuxtCompilerOptions(rootDir, baseURL, buildAssetsDir, {
    vueVersion: compatibility.vueVersion,
    ...(hasCompatibilityOptions ? { compatibility: compatibilityOptions } : {}),
    mode: compatibility.scriptSetupInStandalone === true ? "function" : void 0,
    ...overrides,
  });
}
function resolveNuxtBridgeOptions(bridge) {
  if (bridge === false)
    return {
      autoImports: false,
      components: false,
      i18n: false,
      stableInjectedKeys: false,
    };
  if (bridge === true || bridge == null) return { ...DEFAULT_NUXT_BRIDGE_OPTIONS };
  return {
    autoImports: bridge.autoImports ?? DEFAULT_NUXT_BRIDGE_OPTIONS.autoImports,
    components: bridge.components ?? DEFAULT_NUXT_BRIDGE_OPTIONS.components,
    i18n: bridge.i18n ?? DEFAULT_NUXT_BRIDGE_OPTIONS.i18n,
    stableInjectedKeys: bridge.stableInjectedKeys ?? DEFAULT_NUXT_BRIDGE_OPTIONS.stableInjectedKeys,
  };
}
function resolveNuxtUnoCssOptions(unocss) {
  if (unocss === false) return false;
  if (unocss === true || unocss == null) return { ...DEFAULT_NUXT_UNOCSS_OPTIONS };
  const originalSource = unocss.originalSource;
  if (originalSource === false) return { originalSource: false };
  if (originalSource === true || originalSource == null) return { originalSource: {} };
  return { originalSource };
}
function resolveNuxtDevOptions(dev) {
  return {
    ...DEFAULT_NUXT_DEV_OPTIONS,
    ...dev,
  };
}
function resolveNuxtMuseaOptions(musea) {
  if (musea === true) return {};
  if (musea === false || musea == null) return false;
  return musea;
}
//#endregion
//#region src/resolver.ts
const nodeRequire = createRequire(`${process.cwd()}/package.json`);
function createNuxtModuleResolver() {
  const moduleDir = dirname(nodeRequire.resolve("@vizejs/nuxt"));
  return { resolve: (...segments) => resolve(moduleDir, ...segments) };
}
function readTextFile(filePath) {
  return fs.readFileSync(filePath, "utf-8");
}
function readFileSize(filePath) {
  return fs.statSync(filePath).size;
}
function appendOriginalVueSourceForUnoCss(code, normalizedId, options = {}) {
  const filePath = normalizedId.split("?")[0];
  if (!filePath) return code;
  const maxBytes = Math.max(1, Math.floor(options.maxBytes ?? 2097152));
  const readSize = options.readSize ?? readFileSize;
  const readFile = options.readFile ?? readTextFile;
  try {
    if (readSize(filePath) > maxBytes) return code;
  } catch {
    return code;
  }
  try {
    return `${code}\n${readFile(filePath)}`;
  } catch {
    return code;
  }
}
//#endregion
//#region src/index.ts
const VIZE_NUXT_AUTO_IMPORT_PATCHED = "__vizeNuxtAutoImportPatched";
const VUE_RUNTIME_DEDUPE = [
  "vue",
  "@vue/reactivity",
  "@vue/runtime-core",
  "@vue/runtime-dom",
  "@vue/shared",
];
const moduleMeta = {
  name: "@vizejs/nuxt",
  configKey: "vize",
};
const moduleDefaults = {
  musea: false,
  nuxtMusea: { route: { path: "/" } },
};
let nuxtKitPromise = null;
function loadNuxtKit() {
  nuxtKitPromise ||= import("@nuxt/kit");
  return nuxtKitPromise;
}
async function addNuxtServerPlugin(plugin) {
  const { addServerPlugin } = await loadNuxtKit();
  addServerPlugin(plugin);
}
async function addNuxtVitePlugin(plugin) {
  const { addVitePlugin } = await loadNuxtKit();
  addVitePlugin(plugin);
}
function isPlainRecord(value) {
  return value != null && typeof value === "object" && !Array.isArray(value);
}
function mergePlainRecords(...values) {
  const result = {};
  for (const value of values) {
    if (!value) continue;
    for (const [key, nextValue] of Object.entries(value)) {
      const currentValue = result[key];
      result[key] =
        isPlainRecord(currentValue) && isPlainRecord(nextValue)
          ? mergePlainRecords(currentValue, nextValue)
          : nextValue;
    }
  }
  return result;
}
function resolveModuleOptions(inlineOptions, nuxt) {
  return mergePlainRecords(moduleDefaults, nuxt?.options.vize, inlineOptions);
}
function markNuxtRequiredModule(nuxt) {
  nuxt.options._requiredModules ||= {};
  nuxt.options._requiredModules[moduleMeta.name] = true;
}
function registerNuxt2CompatibilityHooks(nuxt) {
  if (getDetectedNuxtMajor(nuxt) !== 2) return;
  nuxt.hook("close", () => {});
  nuxt.hook("builder:prepared", () => {});
  nuxt.hook("build:templates", () => {});
}
function getDetectedNuxtMajor(nuxt) {
  const nuxtLike = nuxt;
  const version =
    nuxtLike?._version ??
    nuxtLike?.version ??
    (typeof nuxtLike?.options?._nuxtVersion === "string" ? nuxtLike.options._nuxtVersion : null);
  if (!version) return null;
  const major = Number.parseInt(version.split(".")[0] ?? "", 10);
  return major === 2 || major === 3 || major === 4 ? major : null;
}
function hasNuxtViteCompilerSupport(nuxt) {
  const builder = nuxt.options.builder;
  if (typeof builder === "string") return builder === "vite" || builder.includes("vite-builder");
  if (nuxt.options.vite) return true;
  return getDetectedNuxtMajor(nuxt) !== 2;
}
function getNuxtAppBaseURL(nuxt) {
  return nuxt.options.app?.baseURL ?? nuxt.options.router?.base;
}
function getNuxtBuildAssetsDir(nuxt) {
  return nuxt.options.app?.buildAssetsDir ?? nuxt.options.build?.publicPath;
}
function shouldUseVizeCompiler(compilerOptions) {
  return (
    compilerOptions !== false &&
    compilerOptions.compatibility?.hostCompiler !== true &&
    (compilerOptions.vueVersion ?? 3) === 3
  );
}
function dedupeVueRuntimePackages(vite) {
  vite.resolve ||= {};
  const dedupe = new Set(vite.resolve.dedupe ?? []);
  for (const packageName of VUE_RUNTIME_DEDUPE) dedupe.add(packageName);
  vite.resolve.dedupe = [...dedupe];
}
function isViteSsrTransform(args) {
  const options = args[0];
  return (
    typeof options === "object" && options !== null && "ssr" in options && options.ssr === true
  );
}
function normalizeNuxtKeyedTransformResult(id, result) {
  if (!isVizeVirtualVueModuleId(id) || result == null) return result;
  if (typeof result === "string") return normalizeNuxtInjectedKeysForVizeVirtualModule(result, id);
  if (typeof result.code !== "string") return result;
  const code = normalizeNuxtInjectedKeysForVizeVirtualModule(result.code, id);
  return code === result.code
    ? result
    : {
        ...result,
        code,
      };
}
function patchNuxtKeyedFunctionsPlugin(plugin) {
  if (typeof plugin.transform === "function") {
    const original = plugin.transform;
    plugin.transform = async function (code, id, ...args) {
      return normalizeNuxtKeyedTransformResult(id, await original.call(this, code, id, ...args));
    };
    return;
  }
  const transform = plugin.transform;
  if (!transform || typeof transform.handler !== "function") return;
  const original = transform.handler;
  transform.handler = async function (code, id, ...args) {
    return normalizeNuxtKeyedTransformResult(id, await original.call(this, code, id, ...args));
  };
}
function normalizeNuxtAutoImportTransformResult(code, id, result, rewriteVueRuntimeImports) {
  if (!isVizeGeneratedVueModuleId(id) || result == null) return result;
  if (typeof result === "string") {
    const restored = preserveExplicitVueImportsFromVizeModuleSource(
      id,
      preserveExplicitVueImportsFromNuxtAutoImports(code, result),
    );
    return rewriteVueRuntimeImports ? rewriteBareVueImportsToClientRuntime(restored) : restored;
  }
  if (typeof result.code !== "string") return result;
  let normalized = preserveExplicitVueImportsFromVizeModuleSource(
    id,
    preserveExplicitVueImportsFromNuxtAutoImports(code, result.code),
  );
  if (rewriteVueRuntimeImports) normalized = rewriteBareVueImportsToClientRuntime(normalized);
  return normalized === result.code
    ? result
    : {
        ...result,
        code: normalized,
      };
}
function patchNuxtAutoImportTransformPlugin(plugin, isBuild) {
  if (!plugin) return;
  if (plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED]) return;
  if (typeof plugin.transform === "function") {
    const original = plugin.transform;
    plugin.transform = async function (code, id, ...args) {
      return normalizeNuxtAutoImportTransformResult(
        code,
        id,
        await original.call(this, code, id, ...args),
        isBuild && !isViteSsrTransform(args),
      );
    };
    plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED] = true;
    return;
  }
  const transform = plugin.transform;
  if (!transform || typeof transform.handler !== "function") return;
  const original = transform.handler;
  transform.handler = async function (code, id, ...args) {
    return normalizeNuxtAutoImportTransformResult(
      code,
      id,
      await original.call(this, code, id, ...args),
      isBuild && !isViteSsrTransform(args),
    );
  };
  plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED] = true;
}
async function setupVizeNuxtModule(options, nuxt) {
  const resolver = createNuxtModuleResolver();
  const detectedNuxtMajor = options.compatibility?.nuxtVersion ?? getDetectedNuxtMajor(nuxt) ?? 3;
  const vueVersion = options.compatibility?.vueVersion ?? (detectedNuxtMajor === 2 ? 2 : 3);
  const nuxtWithBuilderOptions = nuxt;
  const supportsViteCompiler = hasNuxtViteCompilerSupport(nuxtWithBuilderOptions);
  const appBaseURL = getNuxtAppBaseURL(nuxtWithBuilderOptions);
  const buildAssetsDir = getNuxtBuildAssetsDir(nuxtWithBuilderOptions);
  const bridgeOptions = resolveNuxtBridgeOptions(options.bridge);
  const devOptions = resolveNuxtDevOptions(options.dev);
  const museaOptions = detectedNuxtMajor === 2 ? false : resolveNuxtMuseaOptions(options.musea);
  const unocssOptions = resolveNuxtUnoCssOptions(options.unocss);
  if (museaOptions !== false) nuxt.hook("components:dirs", appendMuseaArtComponentIgnore);
  const compilerOptions = resolveNuxtCompilerOptions(
    nuxt.options.rootDir,
    appBaseURL,
    buildAssetsDir,
    options.compiler,
    {
      supportsViteCompiler,
      vueVersion,
    },
  );
  const usesVizeCompiler = shouldUseVizeCompiler(compilerOptions);
  if (compilerOptions !== false && compilerOptions.compatibility?.hostCompiler !== true) {
    const { default: vize } = await import("@vizejs/vite-plugin");
    nuxt.options.vite ||= {};
    nuxt.options.vite.plugins = nuxt.options.vite.plugins || [];
    nuxt.options.vite.plugins.push(vize(compilerOptions));
  }
  let isNuxtBuild = false;
  let isViteBuild = false;
  if (usesVizeCompiler)
    nuxt.hook("build:before", () => {
      if (nuxt.options.dev !== false) return;
      isNuxtBuild = true;
      nuxt.options.vite ||= {};
      dedupeVueRuntimePackages(nuxt.options.vite);
    });
  if (usesVizeCompiler) {
    if (nuxt.options.dev && devOptions.stylesheetLinks) {
      const devAssetBase =
        compilerOptions.devUrlBase ?? buildNuxtDevAssetBase(appBaseURL, buildAssetsDir);
      nuxt.options.nitro ||= {};
      nuxt.options.nitro.virtual ||= {};
      if (nuxt.options.nitro.virtual) {
        nuxt.options.nitro.virtual["#vizejs/nuxt/dev-stylesheet-links-config"] =
          `export const devAssetBase = ${JSON.stringify(devAssetBase)};`;
        await addNuxtServerPlugin(resolver.resolve("./runtime/server/dev-stylesheet-links"));
      }
    }
    nuxt.hook("vite:configResolved", (config) => {
      isViteBuild = config.command === "build" || isNuxtBuild || nuxt.options.dev === false;
      for (let i = config.plugins.length - 1; i >= 0; i--) {
        const p = config.plugins[i];
        const name = p && typeof p === "object" && "name" in p ? p.name : "";
        if (name === "vite:vue") config.plugins.splice(i, 1);
        else if (bridgeOptions.stableInjectedKeys && name === "nuxt:compiler:keyed-functions")
          patchNuxtKeyedFunctionsPlugin(p);
        if (bridgeOptions.autoImports) patchNuxtAutoImportTransformPlugin(p, isViteBuild);
      }
    });
  }
  let unimportCtx = null;
  if (usesVizeCompiler && bridgeOptions.autoImports)
    nuxt.hook("imports:context", (ctx) => {
      unimportCtx = ctx;
    });
  const nuxtComponentResolver =
    usesVizeCompiler && bridgeOptions.components
      ? createNuxtComponentResolver({
          buildDir: nuxt.options.buildDir,
          moduleNames: nuxt.options.modules.filter((moduleName) => typeof moduleName === "string"),
          rootDir: nuxt.options.rootDir,
        })
      : null;
  if (nuxtComponentResolver)
    nuxt.hook("components:extend", (comps) => {
      nuxtComponentResolver.register(comps);
    });
  if (
    usesVizeCompiler &&
    (bridgeOptions.autoImports ||
      bridgeOptions.components ||
      bridgeOptions.i18n ||
      bridgeOptions.stableInjectedKeys)
  )
    await addNuxtVitePlugin({
      name: "vizejs:nuxt-transform-bridge",
      enforce: "post",
      async transform(code, id, ...args) {
        if (!isVizeGeneratedVueModuleId(id) && !isVizeJsxModuleId(id)) return;
        let result = code;
        let changed = false;
        if (nuxtComponentResolver && hasComponentBridgeInput(result)) {
          const nextComponentResult = injectNuxtComponentImports(result, (name) => {
            return nuxtComponentResolver.resolve(name);
          });
          if (nextComponentResult !== result) {
            result = nextComponentResult;
            changed = true;
          }
        }
        if (bridgeOptions.i18n && hasI18nBridgeInput(result)) {
          const nextResult = injectNuxtI18nHelpers(result, id);
          if (nextResult !== result) {
            result = nextResult;
            changed = true;
          }
        }
        if (unimportCtx)
          try {
            const beforeUnimport = result;
            const injected = await unimportCtx.injectImports(result, id);
            if (injected.imports && injected.imports.length > 0) {
              result = preserveExplicitVueImportsFromNuxtAutoImports(beforeUnimport, injected.code);
              changed = true;
            }
          } catch {}
        if (bridgeOptions.autoImports) {
          const nextResult = preserveExplicitVueImportsFromVizeModuleSource(id, result);
          if (nextResult !== result) {
            result = nextResult;
            changed = true;
          }
        }
        if (bridgeOptions.stableInjectedKeys && hasStableKeyBridgeInput(result)) {
          const stableKeyResult = stabilizeNuxtInjectedKeysForVizeVirtualModule(result, id);
          if (stableKeyResult !== result) {
            result = stableKeyResult;
            changed = true;
          }
        }
        if (isViteBuild && !isViteSsrTransform(args)) {
          const clientRuntimeResult = rewriteBareVueImportsToClientRuntime(result);
          if (clientRuntimeResult !== result) {
            result = clientRuntimeResult;
            changed = true;
          }
        }
        if (changed)
          return {
            code: result,
            map: null,
          };
      },
    });
  if (usesVizeCompiler && unocssOptions !== false)
    await addNuxtVitePlugin({
      name: "vizejs:unocss-bridge",
      configResolved(config) {
        for (const plugin of config.plugins)
          if (plugin.name?.startsWith("unocss:") && typeof plugin.transform === "function") {
            const origTransform = plugin.transform;
            const isExtractionOnly = plugin.name.startsWith("unocss:global");
            plugin.transform = function (code, id, ...args) {
              if (isVizeVirtualVueModuleId(id)) {
                const normalizedId = normalizeVizeVirtualVueModuleId(id);
                let effectiveCode = code;
                if (isExtractionOnly && unocssOptions.originalSource !== false)
                  effectiveCode = appendOriginalVueSourceForUnoCss(code, normalizedId, {
                    maxBytes: unocssOptions.originalSource.maxBytes,
                  });
                return origTransform.call(this, effectiveCode, normalizedId, ...args);
              }
              return origTransform.call(this, code, id, ...args);
            };
          }
      },
    });
  if (museaOptions !== false && supportsViteCompiler) {
    const { musea } = await import("@vizejs/vite-plugin-musea");
    const museaBasePath = "basePath" in museaOptions ? museaOptions.basePath : "/__musea__";
    (nuxt.options.vite ||= {}).plugins ||= [];
    registerNuxtMuseaStaticPublicAsset(nuxt, museaBasePath);
    const museaConfig = {
      projectRoot: nuxt.options.rootDir,
      vueVersion,
      ...museaOptions,
    };
    nuxt.options.vite.plugins.push(...musea(museaConfig));
    nuxt.hook("listen", (_server, listener) => {
      const url = listener.url?.replace(/\/$/, "") || "http://localhost:3000";
      console.log(
        `  \x1b[36m➜\x1b[0m  \x1b[1mMusea Gallery:\x1b[0m \x1b[36m${url}${museaBasePath}\x1b[0m`,
      );
    });
  }
}
const vizeNuxtModule = Object.assign(
  async function vizeNuxtModule(inlineOptions = {}, nuxtArg) {
    const nuxt = nuxtArg ?? this?.nuxt;
    if (!nuxt) throw new Error("@vizejs/nuxt requires a Nuxt instance");
    markNuxtRequiredModule(nuxt);
    registerNuxt2CompatibilityHooks(nuxt);
    await setupVizeNuxtModule(resolveModuleOptions(inlineOptions, nuxt), nuxt);
  },
  {
    getMeta: () => moduleMeta,
    getOptions: (inlineOptions = {}, nuxt) => resolveModuleOptions(inlineOptions, nuxt),
    meta: moduleMeta,
    defaults: moduleDefaults,
  },
);
//#endregion
export { vizeNuxtModule as default };
