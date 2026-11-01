import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpus = path.join(
  root,
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership",
);
const declarationPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
const carrier = "$InstanceRawPropsResult";
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

interface WholeText {
  beforeWholeUtf8: string;
  afterWholeUtf8: string;
  beforeBytes: number;
  afterBytes: number;
  beforeSha256: string;
  afterSha256: string;
}

function moduleAliases(source: string): readonly ts.TypeAliasDeclaration[] {
  const file = ts.createSourceFile("component-checks.d.ts", source, ts.ScriptTarget.Latest, true);
  assert.equal(file.statements.length, 1);
  const module = file.statements[0];
  assert.ok(module && ts.isModuleDeclaration(module));
  assert.ok(module.body && ts.isModuleBlock(module.body));
  assert.ok(
    module.body.statements.every(
      (node) => ts.isTypeAliasDeclaration(node) || ts.isExportDeclaration(node),
    ),
  );
  return module.body.statements.filter(ts.isTypeAliasDeclaration);
}

function retainedGenericStrings(): { contracts: string; genericConsumer: string } {
  const file = ts.createSourceFile(
    "controls.ts",
    readFileSync(path.join(root, "tests/tooling/canon-instance-prop-metadata.test.ts"), "utf8"),
    ts.ScriptTarget.Latest,
    true,
  );
  const result = new Map<string, string>();
  for (const statement of file.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (
        ts.isIdentifier(declaration.name) &&
        declaration.initializer &&
        ts.isNoSubstitutionTemplateLiteral(declaration.initializer)
      ) {
        result.set(declaration.name.text, declaration.initializer.text);
      }
    }
  }
  const contracts = result.get("contracts");
  const genericConsumer = result.get("genericConsumer");
  assert.ok(contracts && genericConsumer);
  return { contracts, genericConsumer };
}

function observe(preamble: string, source: string) {
  // This workspace owns the locked Vue and Vite declarations used by the whole preamble.
  const folder = path.join(root, "npm/builder/vite");
  const helper = path.join(folder, "__vize_instance_prop_ambient.d.ts");
  const consumer = path.join(folder, "__vize_instance_prop_consumer.ts");
  const options: ts.CompilerOptions = {
    noEmit: true,
    strict: true,
    skipLibCheck: true,
    types: [],
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
  };
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  const exists = host.fileExists.bind(host);
  host.readFile = (file) => (file === helper ? preamble : file === consumer ? source : read(file));
  host.fileExists = (file) => file === helper || file === consumer || exists(file);
  const providers = ["vue", "vite/client"].map((name) => {
    const module = ts.resolveModuleName(name, helper, options, host).resolvedModule;
    assert.ok(module, `locked declaration provider is required: ${name}`);
    const bytes = readFileSync(module.resolvedFileName);
    return { name, ...module, bytes: bytes.length, sha256: hash(bytes) };
  });
  const program = ts.createProgram([helper, consumer], options, host);
  function diagnostic(row: ts.Diagnostic): unknown {
    return {
      ...row,
      file: row.file?.fileName,
      relatedInformation: row.relatedInformation?.map(diagnostic),
    };
  }
  const diagnostics = ts.getPreEmitDiagnostics(program).map(diagnostic);
  const checker = program.getTypeChecker();
  const module = checker.getAmbientModules().find((symbol) => symbol.name === '"vize:virtual-ts"');
  assert.ok(module);
  const exports = checker.getExportsOfModule(module).map((symbol) => ({
    name: symbol.name,
    flags: symbol.flags,
    declarations: symbol.declarations?.map((declaration) => ({
      kind: ts.SyntaxKind[declaration.kind],
      file: declaration.getSourceFile().fileName,
      name: ts.getNameOfDeclaration(declaration)?.getText(declaration.getSourceFile()),
      start: declaration.getStart(declaration.getSourceFile()),
      end: declaration.end,
      modifiers: ts.canHaveModifiers(declaration)
        ? ts.getModifiers(declaration)?.map((modifier) => ts.SyntaxKind[modifier.kind])
        : undefined,
    })),
  }));
  return { diagnostics, exports, providers };
}

test("the complete ambient module preserves its public exports and authored augmentation scope", () => {
  const frozenBytes = readFileSync(path.join(corpus, "slot-provider-payload-update-11027.json"));
  assert.equal(
    hash(frozenBytes),
    "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c",
  );
  const frozen: { logicalPreamble: { afterWholeUtf8: string } } = JSON.parse(
    frozenBytes.toString(),
  );
  const update: { source: WholeText; preamble: WholeText } = JSON.parse(
    readFileSync(path.join(corpus, "instance-prop-metadata-update.json"), "utf8"),
  );
  const current = readFileSync(path.join(root, declarationPath), "utf8");
  const old = update.source.beforeWholeUtf8;
  for (const record of [update.source, update.preamble]) {
    for (const role of ["before", "after"] as const) {
      const text = record[`${role}WholeUtf8`];
      assert.equal(Buffer.byteLength(text), record[`${role}Bytes`]);
      assert.equal(hash(text), record[`${role}Sha256`]);
    }
  }
  assert.equal(current, update.source.afterWholeUtf8);
  const before = frozen.logicalPreamble.afterWholeUtf8;
  assert.equal(before, update.preamble.beforeWholeUtf8);
  assert.equal(before.split(old).length, 2);
  const after = before.replace(old, current);
  assert.equal(after, update.preamble.afterWholeUtf8);
  const oldAliases = moduleAliases(old);
  const currentAliases = moduleAliases(current);
  assert.equal(oldAliases.length, 44);
  assert.equal(currentAliases.length, 45);
  for (const original of oldAliases) {
    const actual = currentAliases.find((node) => node.name.text === original.name.text);
    assert.ok(actual);
    assert.ok(actual.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword));
    if (!["$InstanceRawProps", "$HasInstanceRawProps"].includes(original.name.text)) {
      assert.equal(
        actual.getText().replace(/^export /u, ""),
        original.getText().replace(/^export /u, ""),
      );
    }
  }
  const privateAlias = currentAliases.find((node) => node.name.text === carrier);
  assert.ok(privateAlias && !privateAlias.modifiers?.length);
  const { contracts, genericConsumer } = retainedGenericStrings();
  const augment = `declare module "vize:virtual-ts" { export type ${carrier}<C> = "authored"; }\n`;
  const cases = {
    augmentation: `import type {} from "vize:virtual-ts";\n${augment}type Probe = import("vize:virtual-ts").${carrier}<{}>;\nconst result: Probe = "authored";\n`,
    importNew: `import type { ${carrier} } from "vize:virtual-ts";\ntype Probe = ${carrier}<{}>;\nconst result: Probe = { present: false, props: {} };\n`,
    empty: 'import type {} from "vize:virtual-ts";\nexport {};\n',
    oldImports: `import type { ${oldAliases.map((node) => node.name.text).join(", ")} } from "vize:virtual-ts";\nexport {};\n`,
    oldAugment:
      'import type {} from "vize:virtual-ts";\ndeclare module "vize:virtual-ts" {\n' +
      oldAliases
        .map(
          (node) =>
            `export type ${node.name.text}${node.typeParameters?.length ? "<" + node.typeParameters.map((parameter) => parameter.name.text).join(", ") + ">" : ""} = "authored";`,
        )
        .join("\n") +
      "\n}\n",
    deferred: `import type * as Subject from "vize:virtual-ts";\n${contracts}\n${genericConsumer}`,
    augmentedDeferred: `import type * as Subject from "vize:virtual-ts";\n${contracts}\n${genericConsumer}\n${augment}`,
  };
  const observations = Object.entries(cases).map(([id, source]) => ({
    id,
    source,
    sourceSha256: hash(source),
    before: observe(before, source),
    after: observe(after, source),
  }));
  assert.equal(current.split("export {};\n").length, 2);
  const leaking = after.replace(current, current.replace("export {};\n", ""));
  const negative = {
    augmentation: observe(leaking, cases.augmentation),
    importNew: observe(leaking, cases.importNew),
  };
  // All outcomes, including the export-context mutation, precede semantic verdicts.
  for (const row of observations)
    console.log(
      "canon-instance-prop-ambient:",
      JSON.stringify({
        typescript: ts.version,
        beforeSha256: hash(before),
        afterSha256: hash(after),
        ...row,
      }),
    );
  console.log("canon-instance-prop-ambient-negative:", JSON.stringify(negative));
  const expectedCounts = [0, 1, 0, 0, 1, 5, 5];
  for (const [index, row] of observations.entries()) {
    assert.equal(row.before.diagnostics.length, expectedCounts[index]);
    assert.deepEqual(row.after.diagnostics, row.before.diagnostics);
    const roster = (value: ReturnType<typeof observe>) =>
      value.exports.map(({ name, flags, declarations }) => ({
        name,
        flags,
        declarations: declarations?.map(({ kind, file, name }) => ({ kind, file, name })),
      }));
    assert.deepEqual(roster(row.after), roster(row.before));
    assert.deepEqual(row.after.providers, row.before.providers);
  }
  assert.equal(observations[2]?.before.exports.length, 78);
  assert.equal(negative.importNew.diagnostics.length, 0);
  assert.notDeepEqual(negative.augmentation.diagnostics, observations[0]?.before.diagnostics);
});
