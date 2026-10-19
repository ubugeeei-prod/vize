import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import path from "node:path";
import process from "node:process";
process.env.NODE_ENV = "production";
async function readStdin() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

async function importFromRoots(specifier, roots) {
  for (const root of roots) {
    if (!root) continue;
    try {
      const require = createRequire(path.join(root, "package.json"));
      const resolved = require.resolve(specifier);
      return await import(pathToFileURL(resolved).href);
    } catch {
      // Try the next root, then the default resolver below.
    }
  }
  return await import(specifier);
}

function normalizeCompilerMessages(messages) {
  return (messages ?? []).map((message) => {
    if (message instanceof Error) return message.message;
    if (typeof message === "object" && message && "message" in message) {
      return String(message.message);
    }
    return String(message);
  });
}

function descriptorUsesTypeScript(descriptor) {
  const langs = [descriptor.script?.lang, descriptor.scriptSetup?.lang];
  return langs.some((lang) => lang === "ts" || lang === "tsx");
}

function toErrorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

// CommonJS bundles whose exports Node cannot detect expose their API only as `default`.
function moduleApi(namespace, member) {
  return member in namespace ? namespace : (namespace.default ?? namespace);
}

async function loadFormatter(roots) {
  try {
    const [prettier, parserBabel, parserEstree, parserTypescript] = await Promise.all([
      importFromRoots("prettier/standalone", roots),
      importFromRoots("prettier/plugins/babel", roots),
      importFromRoots("prettier/plugins/estree", roots),
      importFromRoots("prettier/plugins/typescript", roots),
    ]);
    return {
      prettier: moduleApi(prettier, "format"),
      parserBabel: moduleApi(parserBabel, "parsers"),
      parserEstree: moduleApi(parserEstree, "printers"),
      parserTypescript: moduleApi(parserTypescript, "parsers"),
    };
  } catch {
    return null;
  }
}

async function formatCode(source, parser, formatter) {
  if (!source || !formatter) return source ?? "";
  const plugins =
    parser === "typescript"
      ? [formatter.parserTypescript, formatter.parserEstree]
      : [formatter.parserBabel, formatter.parserEstree];
  try {
    return await formatter.prettier.format(source, {
      parser,
      plugins,
      semi: true,
      singleQuote: false,
      printWidth: 100,
    });
  } catch {
    return source;
  }
}

async function compileOfficialVue(file, target, compiler, formatter) {
  const start = performance.now();
  try {
    const officialTarget = target === "ssr" ? "ssr" : "dom";
    const parsed = compiler.parse(file.source, { filename: file.path });
    const descriptor = parsed.descriptor;
    const isTypeScript = descriptorUsesTypeScript(descriptor);
    const parser = isTypeScript ? "typescript" : "babel";
    const warnings = normalizeCompilerMessages(parsed.errors);
    let bindingMetadata = {};
    let scriptCode = "";
    const scoped = descriptor.styles.some((style) => style.scoped);
    const inlineTemplate = Boolean(
      descriptor.scriptSetup && descriptor.template && officialTarget === "dom",
    );

    if (descriptor.script || descriptor.scriptSetup) {
      const script = compiler.compileScript(descriptor, {
        id: file.path,
        inlineTemplate,
        isProd: true,
        templateOptions: inlineTemplate
          ? {
              filename: file.path,
              id: file.path,
              scoped,
              slotted: descriptor.slotted,
              isProd: true,
              compilerOptions: {
                expressionPlugins: isTypeScript ? ["typescript"] : undefined,
              },
            }
          : { ssr: officialTarget === "ssr" },
      });
      scriptCode = script.content;
      bindingMetadata = script.bindings;
    }

    let templateCode = "";
    if (descriptor.template && !inlineTemplate) {
      const template = compiler.compileTemplate({
        source: descriptor.template.content,
        filename: file.path,
        id: file.path,
        scoped,
        slotted: descriptor.slotted,
        isProd: true,
        ssr: officialTarget === "ssr",
        ssrCssVars: descriptor.cssVars,
        compilerOptions: {
          bindingMetadata,
          expressionPlugins: isTypeScript ? ["typescript"] : undefined,
        },
      });
      templateCode = template.code;
      warnings.push(...normalizeCompilerMessages(template.errors));
      warnings.push(...normalizeCompilerMessages(template.tips));
    }

    const code = [scriptCode, templateCode].filter(Boolean).join("\n\n");
    return {
      code,
      formattedCode: await formatCode(code, parser, formatter),
      warnings,
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      code: "",
      formattedCode: "",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

function parserForFile(file, compiler) {
  try {
    const parsed = compiler.parse(file.source, { filename: file.path });
    return descriptorUsesTypeScript(parsed.descriptor) ? "typescript" : "babel";
  } catch {
    return "babel";
  }
}

const input = JSON.parse(await readStdin());
const roots = [process.cwd(), input.moduleRoot].filter(Boolean);
const compiler = await importFromRoots("vue/compiler-sfc", roots);
const formatter = await loadFormatter(roots);
const files = [];

for (const file of input.files) {
  const parser = parserForFile(file, compiler);
  const official = await compileOfficialVue(file, input.target, compiler, formatter);
  files.push({
    path: file.path,
    parser,
    official,
    vizeFormattedCode: await formatCode(file.vizeCode, parser, formatter),
  });
}

process.stdout.write(JSON.stringify({ files }));
