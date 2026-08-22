import assert from "node:assert/strict";
import { test } from "node:test";

import {
  type PinnedFixtureWorkspace,
  withPinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveTsgoBinary,
  runVizeCheck,
} from "../../_helpers/realworld-typecheck.ts";

const FIXTURE_ID = "mobile-web-best-practice";
const CARD_PATH = "src/views/home/widgets/card.vue";
const CLASS_ANCHOR = "export default class Card extends Vue {\n";
const BROKEN_CLASS_ANCHOR = `${CLASS_ANCHOR}  private vizeLegacyProbe = missingVizeLegacyProbe;\n`;
const BROKEN_DIAGNOSTIC = "error:71:29 [TS2304] Cannot find name 'missingVizeLegacyProbe'.";

test("pinned Vue 2 class-component app detects and repairs an exact authored error", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: FIXTURE_ID, includePaths: ["package.json", CARD_PATH, "src/types"] },
    async (fixture) => {
      configureWorkspace(fixture, corsaPath);
      assertPinnedVue2ClassStack(fixture);

      const pinnedSource = fixture.read(CARD_PATH);
      const clean = runVizeCheck(fixture.workspaceDir, corsaPath, [CARD_PATH]);
      assert.equal(clean.status, 0, clean.stderr || clean.stdout);
      assert.equal(clean.stderr, "");
      assert.equal(clean.report.fileCount, 1);
      assert.equal(clean.report.errorCount, 0);
      assert.deepEqual(diagnosticsFor(clean.report.files), []);

      const brokenSource = fixture.applyExactPatch(CARD_PATH, CLASS_ANCHOR, BROKEN_CLASS_ANCHOR);
      const brokenFirst = runVizeCheck(fixture.workspaceDir, corsaPath, [CARD_PATH]);
      const brokenSecond = runVizeCheck(fixture.workspaceDir, corsaPath, [CARD_PATH]);
      assert.equal(brokenFirst.status, 1, brokenFirst.stderr || brokenFirst.stdout);
      assert.equal(brokenFirst.stderr, "");
      assert.equal(brokenSecond.stdout, brokenFirst.stdout, "broken JSON must be byte-stable");
      assert.equal(fixture.read(CARD_PATH), brokenSource, "check must preserve the broken edit");
      assert.deepEqual(diagnosticsFor(brokenFirst.report.files), [BROKEN_DIAGNOSTIC]);

      const repairedSource = fixture.applyExactPatch(CARD_PATH, BROKEN_CLASS_ANCHOR, CLASS_ANCHOR);
      assert.equal(repairedSource, pinnedSource, "repair must restore the exact pinned source");
      const repaired = runVizeCheck(fixture.workspaceDir, corsaPath, [CARD_PATH]);
      assert.equal(repaired.status, 0, repaired.stderr || repaired.stdout);
      assert.deepEqual(omitProgramEvidence(repaired.report), omitProgramEvidence(clean.report));
      assert.equal(repaired.stdout, clean.stdout, "repair must restore byte-stable JSON");
    },
  );
});

function configureWorkspace(fixture: PinnedFixtureWorkspace, corsaPath: string): void {
  fixture.write(
    "node_modules/vue/package.json",
    json({ name: "vue", version: "2.6.10", types: "index.d.ts" }),
  );
  fixture.write(
    "node_modules/vue/index.d.ts",
    `declare class Vue {
  static use(plugin: unknown): void;
  $emit(event: string, ...args: unknown[]): void;
}
export { Vue };
export default Vue;
`,
  );
  fixture.write(
    "node_modules/vue-property-decorator/package.json",
    json({ name: "vue-property-decorator", version: "8.1.0", types: "index.d.ts" }),
  );
  fixture.write(
    "node_modules/vue-property-decorator/index.d.ts",
    `import Vue from "vue";
export { Vue };
export declare function Component(options?: object): ClassDecorator;
export declare function Prop(options?: object): PropertyDecorator;
`,
  );
  fixture.write(
    "node_modules/vuedraggable/package.json",
    json({ name: "vuedraggable", version: "2.23.2", types: "index.d.ts" }),
  );
  fixture.write(
    "node_modules/vuedraggable/index.d.ts",
    "declare const draggable: unknown;\nexport default draggable;\n",
  );
  fixture.write(
    "node_modules/vant/package.json",
    json({ name: "vant", version: "2.1.2", types: "index.d.ts" }),
  );
  fixture.write("node_modules/vant/index.d.ts", "export declare const Checkbox: unknown;\n");
  fixture.write(
    "tsconfig.json",
    json({
      compilerOptions: {
        experimentalDecorators: true,
        lib: ["ES2022", "DOM"],
        module: "ESNext",
        moduleResolution: "Bundler",
        paths: { "@/*": ["./src/*"] },
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
        useDefineForClassFields: false,
      },
      include: ["src/**/*.d.ts", CARD_PATH],
    }),
  );
  fixture.write(
    "vize.config.json",
    json({
      compiler: { compatibility: { vueVersion: "2" } },
      typeChecker: { corsaPath, legacyVue2: true },
    }),
  );
}

function assertPinnedVue2ClassStack(fixture: PinnedFixtureWorkspace): void {
  const manifest = JSON.parse(fixture.read("package.json")) as {
    dependencies?: Record<string, string>;
  };
  assert.deepEqual(
    {
      vue: manifest.dependencies?.vue,
      "vue-class-component": manifest.dependencies?.["vue-class-component"],
      "vue-property-decorator": manifest.dependencies?.["vue-property-decorator"],
    },
    {
      vue: "^2.6.10",
      "vue-class-component": "^7.0.2",
      "vue-property-decorator": "^8.1.0",
    },
    "the pinned upstream must remain a Vue 2-era class-component application",
  );
  assert.match(fixture.read(CARD_PATH), /@Component\([\s\S]*@Prop\(/);
  assert.match(fixture.read(CARD_PATH), /export default class Card extends Vue/);
}

function diagnosticsFor(files: Array<{ diagnostics: string[]; file: string }>): string[] {
  return files.find((entry) => entry.file === CARD_PATH)?.diagnostics ?? [];
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
