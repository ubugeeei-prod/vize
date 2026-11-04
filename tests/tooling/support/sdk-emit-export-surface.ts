import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import ts from "../../../npm/cli/node_modules/typescript/lib/typescript.js";
import {
  assertPublicSlotExportSurface,
  publicSlotExportSurface,
} from "./public-slot-export-surface.ts";

const privateNames = ["$LooseEmitProps", "$LooseEmitPropsFactory", "$EmitIsWide"];
const sha256 = (text: string | Buffer) => createHash("sha256").update(text).digest("hex");
function historicalSources(root: string) {
  const historicalInput = fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json",
    ),
  );
  assert.equal(
    sha256(historicalInput),
    "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c",
  );
  const historical = JSON.parse(historicalInput.toString()) as {
    files: Array<{ path: string; afterWholeUtf8: string }>;
  };
  const oldManifest = fs.readFileSync(
    path.join(root, "tests/fixtures/typechecker/public-slot-instance/package-before.json.txt"),
    "utf8",
  );
  assert.equal(
    sha256(oldManifest),
    "6d4cddbdcd51ccb049a695688b5896294ad6aa23abb812f867e6bf6bd0b50d36",
  );

  return { historical, oldManifest };
}

function stage(
  root: string,
  directory: string,
  before: boolean,
  consumerDependencies: string,
): string {
  const { historical, oldManifest } = historicalSources(root);
  const pkg = path.join(directory, "node_modules/vize");
  const manifest = before
    ? oldManifest
    : fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf8");
  fs.mkdirSync(pkg, { recursive: true });
  fs.writeFileSync(path.join(directory, "package.json"), '{"private":true,"type":"module"}\n');
  fs.writeFileSync(path.join(pkg, "package.json"), manifest);
  const files = before
    ? historical.files.filter(
        (file) => file.path.startsWith("npm/cli/") && /\.d\.(?:ts|mts|cts)$/u.test(file.path),
      )
    : (JSON.parse(manifest) as { files: string[] }).files
        .filter((file) => /\.d\.(?:ts|mts|cts)$/u.test(file))
        .map((file) => ({
          path: "npm/cli/" + file,
          afterWholeUtf8: fs.readFileSync(path.join(root, "npm/cli", file), "utf8"),
        }));
  for (const file of files) {
    const target = path.join(pkg, file.path.slice("npm/cli/".length));
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, file.afterWholeUtf8);
  }
  fs.symlinkSync(consumerDependencies, path.join(pkg, "node_modules"), "dir");
  return pkg;
}

const options: ts.CompilerOptions = {
  strict: true,
  noEmit: true,
  skipLibCheck: true,
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.NodeNext,
  moduleResolution: ts.ModuleResolutionKind.NodeNext,
  types: [],
};

function observe(directory: string, extension: string, direct: boolean) {
  const specifier = direct ? "./node_modules/vize/virtual-ts.js" : "vize/virtual-ts";
  const input = path.join(directory, "consumer." + extension);
  fs.writeFileSync(
    input,
    [
      `import type * as SDK from ${JSON.stringify(specifier)};`,
      `import type { ${privateNames.join(", ")} } from ${JSON.stringify(specifier)};`,
      "export type PublicFactories = [SDK.$NativeEmitPropsFactory<unknown>, SDK.$EmitPropsFactory<unknown>];",
      "export type PrivateProbe = [$LooseEmitProps, $LooseEmitPropsFactory<unknown>, $EmitIsWide<unknown>];",
    ].join("\n"),
  );
  const host = ts.createCompilerHost(options);
  const program = ts.createProgram([input], options, host);
  const source = program.getSourceFile(input)!;
  assert.ok(source);
  const diagnostics = ts
    .getPreEmitDiagnostics(program)
    .filter((diagnostic) => diagnostic.file?.fileName === input)
    .map((diagnostic) => ({
      code: diagnostic.code,
      start: diagnostic.start,
      length: diagnostic.length,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    }));
  return {
    input,
    host,
    program,
    diagnostics,
    surface: publicSlotExportSurface(program.getTypeChecker(), source),
  };
}

function requireDeclarationClosure(observation: ReturnType<typeof observe>, pkg: string) {
  const dependencies = new Set<string>();
  for (const source of observation.program.getSourceFiles()) {
    if (!source.fileName.startsWith(pkg + path.sep)) continue;
    const visit = (node: ts.Node): void => {
      const specifier =
        ts.isImportDeclaration(node) || ts.isExportDeclaration(node)
          ? node.moduleSpecifier
          : ts.isImportTypeNode(node) && ts.isLiteralTypeNode(node.argument)
            ? node.argument.literal
            : undefined;
      if (specifier && ts.isStringLiteral(specifier)) {
        const resolved = ts.resolveModuleName(
          specifier.text,
          source.fileName,
          options,
          observation.host,
          undefined,
          undefined,
          ts.ModuleKind.ESNext,
        ).resolvedModule;
        assert.ok(
          resolved,
          `physical declaration dependency: ${specifier.text} in ${source.fileName}`,
        );
        assert.ok(fs.existsSync(resolved.resolvedFileName));
        assert.ok(observation.program.getSourceFile(resolved.resolvedFileName));
        if (specifier.text.startsWith(".")) {
          assert.ok(
            resolved.resolvedFileName.startsWith(pkg + path.sep),
            "relative declarations stay in the staged published package",
          );
        }
        dependencies.add(specifier.text);
      }
      ts.forEachChild(node, visit);
    };
    visit(source);
  }
  assert.ok(dependencies.has("./virtual-ts-emits.js"));
  assert.ok(dependencies.has("vue"), "the real consumer Vue dependency resolves");
  assert.equal(
    ts.resolveModuleName("vize/virtual-ts-emits", observation.input, options, observation.host)
      .resolvedModule,
    undefined,
    "private declaration is not a NodeNext package subpath export",
  );
  return [...dependencies].sort();
}

export type SdkModuleMode = "esm" | "commonjs" | "direct";

/** Exercise the physical shipped declarations through the real consumer resolver. */
export function verifyEmitExportSurface(
  root: string,
  mode: SdkModuleMode,
  consumerDependencies = path.join(root, "npm/cli/node_modules"),
) {
  const dependencyRoot = fs.realpathSync(consumerDependencies);
  const extension = { esm: "mts", commonjs: "cts", direct: "ts" }[mode];
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "vize-emit-surface-")));
  try {
    const beforeDirectory = path.join(directory, "before"),
      currentDirectory = path.join(directory, "current");
    stage(root, beforeDirectory, true, dependencyRoot);
    const pkg = stage(root, currentDirectory, false, dependencyRoot);
    const before = observe(beforeDirectory, extension, mode === "direct"),
      current = observe(currentDirectory, extension, mode === "direct");
    assert.equal(before.diagnostics.length, 3, "frozen named imports are rejected");
    assert.deepEqual(
      current.diagnostics,
      before.diagnostics,
      "private names retain the complete original refusal messages and positions",
    );
    assertPublicSlotExportSurface(current.surface, before.surface);
    assert.ok(current.program.getSourceFile(path.join(pkg, `virtual-ts.d.${extension}`)));
    for (const name of privateNames)
      assert.ok(!current.surface.exports.some((entry) => entry.name === name));
    const dependencies = requireDeclarationClosure(current, pkg);
    const leaf = current.program.getSourceFile(path.join(pkg, "virtual-ts-emits.d.ts"));
    assert.ok(leaf);
    const checker = current.program.getTypeChecker(),
      symbol = checker.getSymbolAtLocation(leaf);
    assert.ok(symbol);
    assert.deepEqual(
      checker
        .getExportsOfModule(symbol)
        .map((entry) => entry.name)
        .sort(),
      ["$EmitPropsFactory", "$NativeEmitPropsFactory"],
      "the dedicated leaf exposes only the two forwarded aliases",
    );
    const physicalDeclarations = current.program
      .getSourceFiles()
      .filter((source) => source.fileName.startsWith(pkg + path.sep))
      .map((source) => ({
        path: path.relative(pkg, source.fileName),
        sha256: sha256(fs.readFileSync(source.fileName)),
      }));
    return {
      mode,
      typescript: ts.version,
      dependencyRoot,
      before: before.surface,
      current: current.surface,
      diagnostics: current.diagnostics,
      dependencies,
      physicalDeclarations,
    };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
