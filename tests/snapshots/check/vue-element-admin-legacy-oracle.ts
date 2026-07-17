import assert from "node:assert/strict";
import { test } from "node:test";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";

const sourcePath = "src/views/dashboard/admin/components/TransactionTable.vue";
const cleanBinding = ':data="list"';
const brokenBinding = ':data="missingList"';

test("vue-element-admin legacy slot scopes recover exact CLI diagnostics", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("src/api/remote-search.d.ts", remoteSearchDeclaration);
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write(
        "vize.config.json",
        json({
          compiler: { compatibility: { vueVersion: "2" } },
          globalTypes: { toThousandFilter: "any" },
          typeChecker: { corsaPath, legacyVue2: true },
        }),
      );

      const source = fixture.read(sourcePath);
      assertCleanCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

      const brokenSource = fixture.applyExactPatch(sourcePath, cleanBinding, brokenBinding);
      assert.notEqual(source, brokenSource);
      assertBrokenCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

      const repairedSource = fixture.applyExactPatch(sourcePath, brokenBinding, cleanBinding);
      assert.equal(repairedSource, source);
      assertCleanCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));
    },
  );
});

function assertCleanCheck(result: VizeCheckResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(result.report, {
    files: [{ file: sourcePath, diagnostics: [] }],
    errorCount: 0,
    warningCount: 0,
    fileCount: 1,
  });
}

function assertBrokenCheck(result: VizeCheckResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.deepEqual(result.report, {
    files: [
      {
        file: sourcePath,
        diagnostics: ["error:2:20 [TS2304] Cannot find name 'missingList'."],
      },
    ],
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
  });
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

const tsconfig = {
  compilerOptions: {
    allowJs: true,
    baseUrl: ".",
    lib: ["ES2022", "DOM"],
    paths: { "@/*": ["src/*"] },
    skipLibCheck: true,
    strict: false,
  },
  include: ["src"],
};

const remoteSearchDeclaration = `export function transactionList(): Promise<{
  data: {
    items: Array<{ order_no: string; price: number; status: "success" | "pending" }>;
  };
}>;
`;
