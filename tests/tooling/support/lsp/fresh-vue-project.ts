import fs from "node:fs";
import path from "node:path";
import type { TestContext } from "node:test";
import { binaryRelativePath } from "../../../differential/build-receipt.ts";
import { requireTypecheckDependency, resolveTypecheckRuntime } from "../typecheck-dependency.ts";
import { readPolicyFixture } from "./fresh-capability-policy.ts";
import { root, testOutputRoot } from "./paths.ts";
import { LspSession } from "./session.ts";

/** Real Vue declarations are required; a synthetic Vue module cannot qualify this policy. */
export async function withFreshVueProject(
  t: TestContext,
  label: string,
  run: (session: LspSession, workspaceDir: string) => Promise<void>,
): Promise<void> {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "Corsa runtime for fresh Vue JSX type checking",
    "Corsa runtime unavailable",
  );
  if (runtime === undefined) return;
  const vue = requireTypecheckDependency(
    t,
    [
      process.env.VIZE_TEST_VUE_PACKAGE,
      path.join(root, "node_modules/vue"),
      path.join(root, "tests/node_modules/vue"),
      path.join(root, "playground/node_modules/vue"),
      path.join(root, "examples/jsx-tsx/node_modules/vue"),
      path.join(root, "npm/cli/node_modules/vue"),
    ].find((candidate) => candidate !== undefined && fs.existsSync(candidate)),
    "real Vue package for fresh JSX component diagnostics",
    "real Vue package unavailable",
  );
  if (vue === undefined) return;

  const output = path.join(testOutputRoot, "lsp-fresh-capability-policy");
  const workspaces = path.join(output, "workspaces");
  fs.mkdirSync(workspaces, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(workspaces, `${label}-`));
  const session = new LspSession({
    repoRoot: root,
    binary: process.env.VIZE_LSP_BIN ?? path.join(root, binaryRelativePath()),
  });
  const publications: unknown[] = [];
  session.notificationObservers.push((method, params) => {
    publications.push({ method, params });
    fs.writeFileSync(
      path.join(output, `${label}.notifications.json`),
      JSON.stringify(publications, null, 2) + "\n",
    );
  });
  try {
    const source = path.join(workspaceDir, "src");
    fs.mkdirSync(source);
    const nodeModules = path.join(workspaceDir, "node_modules");
    fs.mkdirSync(nodeModules);
    const realVue = fs.realpathSync(vue);
    const kind = process.platform === "win32" ? "junction" : "dir";
    fs.symlinkSync(realVue, path.join(nodeModules, "vue"), kind);
    fs.symlinkSync(path.join(path.dirname(realVue), "@vue"), path.join(nodeModules, "@vue"), kind);
    fs.writeFileSync(path.join(workspaceDir, "package.json"), '{"private":true,"type":"module"}\n');
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          allowJs: true,
          checkJs: true,
          jsx: "preserve",
          jsxImportSource: "vue",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*"],
      }),
    );
    fs.writeFileSync(path.join(source, "Counter.vue"), readPolicyFixture("Counter.vue.txt"));
    await run(session, workspaceDir);
  } finally {
    await session.shutdown();
    fs.writeFileSync(path.join(output, `${label}.stderr.txt`), session.stderrText);
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
}
