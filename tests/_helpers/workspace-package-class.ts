import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import type { VizeCheckJson } from "./vize-check.ts";

export const workspaceProviderFiles = [
  "packages/c-variant/node_modules/__vize_pnpm_fixture_provider/package.json",
  "packages/c-variant/node_modules/__vize_pnpm_fixture_provider/index.d.ts",
];

export function prepareWorkspaceVariantProvider(root: string): void {
  fs.mkdirSync(path.dirname(path.join(root, workspaceProviderFiles[0]!)), { recursive: true });
  fs.writeFileSync(
    path.join(root, workspaceProviderFiles[0]!),
    '{"name":"__vize_pnpm_fixture_provider","types":"./index.d.ts"}\n',
  );
  fs.writeFileSync(
    path.join(root, workspaceProviderFiles[1]!),
    'export declare const authority: "second";\n',
  );
  fs.appendFileSync(
    path.join(root, "packages/c-variant/src/index.ts"),
    'import { authority } from "__vize_pnpm_fixture_provider";\nconst providerMustBeTyped: "second" = authority;\nvoid providerMustBeTyped;\n',
  );
}

export function assertWorkspaceClassConflict(
  receipt: { status: number | null; report: VizeCheckJson },
  expected: string[],
): void {
  const errors = receipt.report.files.find(
    (file) => file.file === "packages/a/src/index.ts",
  )?.diagnostics;
  assert.equal(errors?.length, 1);
  assert.match(errors![0]!, /^error:3:7 \[TS2322\] Type/);
  assert.ok(errors![0]!.includes("Types have separate declarations of a private property 'brand'"));
  assert.deepEqual(
    {
      status: receipt.status,
      errorCount: receipt.report.errorCount,
      warningCount: receipt.report.warningCount,
      fileCount: receipt.report.fileCount,
      files: receipt.report.files.map(({ file, diagnostics }) => ({ file, diagnostics })),
    },
    {
      status: 1,
      errorCount: 1,
      warningCount: 0,
      fileCount: expected.length,
      files: expected.map((file) => ({
        file,
        diagnostics: file === "packages/a/src/index.ts" ? errors : [],
      })),
    },
  );
}

// Secondary module-identity control; the original reporter corpus stays unchanged.
export function prepareWorkspaceClass(root: string): void {
  fs.appendFileSync(
    path.join(root, "packages/c/src/index.ts"),
    "export class Thing { private readonly brand!: void; }\n",
  );
  for (const directory of ["x", "y"]) {
    fs.mkdirSync(path.join(root, "packages/b/src", directory));
  }
  fs.writeFileSync(
    path.join(root, "packages/b/src/x/make.ts"),
    'import { Thing } from "@x/c";\nexport const makeThing = () => new Thing();\n',
  );
  fs.writeFileSync(
    path.join(root, "packages/b/src/y/use.ts"),
    'import { Thing } from "@x/c";\nexport const useThing = (value: Thing) => { void value; };\n',
  );
  fs.writeFileSync(
    path.join(root, "packages/b/src/index.ts"),
    'import { makeThing } from "./x/make";\nimport { useThing } from "./y/use";\nuseThing(makeThing());\nexport { makeThing };\nexport * from "@x/c";\n',
  );
  fs.writeFileSync(
    path.join(root, "packages/a/src/index.ts"),
    'import { Thing } from "@x/c";\nimport { makeThing } from "@x/b";\nconst thing: Thing = makeThing();\nvoid thing;\nexport * from "@x/b";\n',
  );
}
