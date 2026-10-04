import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizeCheckJson } from "./vize-check.ts";

export type WorkspaceReceipt = {
  args: string[];
  execution: number;
  cwd: string;
  cliPid: number;
  cliSha256: string;
  nativeSha256: string;
  sourceSha: string;
  input: Array<{ path: string; sha256: string }>;
  links: Array<{ path: string; target: string }>;
  report: VizeCheckJson;
  status: number | null;
  stderr: string;
  stdout: string;
  cache?: ReturnType<typeof captureWorkspaceInstall>;
};
export const workspaceProviderFiles = [
  "packages/c-variant/node_modules/__vize_pnpm_fixture_provider/package.json",
  "packages/c-variant/node_modules/__vize_pnpm_fixture_provider/index.d.ts",
];

function workspaceInstallSnapshot(root: string) {
  // This oracle runs in the existing Ubuntu full Check lane. Bind the actual
  // documented project-key namespace; no extra CLI invocation or native flag.
  assert.equal(process.platform, "linux");
  const key = createHash("sha256")
    .update("vize-canon-project-key:v1\0")
    .update(fs.realpathSync(root))
    .digest("hex");
  const cache = fs.realpathSync(process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache"));
  const virtualRoot = path.join(cache, "vize/canon/projects", key);
  const selected = path.join(virtualRoot, "packages/b/node_modules/@x/c");
  const metadata = fs.lstatSync(selected);
  const files = [
    "package.json",
    "src/index.ts",
    "src/util.ts",
    "src/Btn.vue.ts",
    "src/Btn.d.vue.ts",
  ].map((relative) => ({
    relative,
    content: fs.readFileSync(path.join(selected, relative), "utf8"),
  }));
  const authored = ["packages/c", "packages/c-variant"].flatMap((directory) =>
    ["package.json", "src/index.ts", "src/util.ts", "src/Btn.vue"].map((relative) => ({
      path: `${directory}/${relative}`,
      content: fs.readFileSync(path.join(root, directory, relative), "utf8"),
    })),
  );
  return {
    virtualRoot,
    selected,
    kind: metadata.isSymbolicLink() ? "link" : metadata.isDirectory() ? "directory" : "other",
    target: metadata.isSymbolicLink() ? fs.readlinkSync(selected) : null,
    canonicalTarget: fs.realpathSync(selected),
    files,
    authored,
  };
}

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

export function captureWorkspaceInstall(root: string) {
  try {
    return workspaceInstallSnapshot(root);
  } catch (error) {
    // Retain the original native process first, even if cache capture fails.
    return { error: String(error) };
  }
}

export function assertWorkspaceInstall(receipt: WorkspaceReceipt): void {
  const snapshot = receipt.cache;
  if (!snapshot) return;
  if ("error" in snapshot) assert.fail(snapshot.error);
  const packageRoot = fs.realpathSync(path.join(receipt.cwd, "packages/b/node_modules/@x/c"));
  const variant = packageRoot === path.join(receipt.cwd, "packages/c-variant");
  assert.equal(snapshot.kind, variant ? "directory" : "link");
  assert.ok(snapshot.canonicalTarget.startsWith(`${snapshot.virtualRoot}${path.sep}`));
  if (variant) {
    assert.equal(snapshot.target, null);
    assert.equal(snapshot.canonicalTarget, snapshot.selected);
  } else {
    assert.ok(snapshot.target?.startsWith(`${snapshot.virtualRoot}${path.sep}`));
    assert.equal(fs.realpathSync(snapshot.target!), snapshot.canonicalTarget);
  }
  for (const relative of ["package.json", "src/index.ts", "src/util.ts"]) {
    assert.equal(
      snapshot.files.find((file) => file.relative === relative)?.content,
      fs.readFileSync(path.join(packageRoot, relative), "utf8"),
    );
  }
  const source = `${variant ? "packages/c-variant" : "packages/c"}/src/Btn.vue`;
  assert.equal(
    snapshot.files.find((file) => file.relative === "src/Btn.vue.ts")?.content,
    receipt.report.files.find((file) => file.file === source)?.virtualTs,
  );
}
