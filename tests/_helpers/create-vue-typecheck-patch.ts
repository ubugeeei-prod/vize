import type { PinnedFixtureWorkspace } from "./realworld-patch.ts";

export const createVueTypecheckAppPath = "template/bare/typescript/src/App.vue";
export const createVueCleanCount = "const count: number = 1";
export const createVueBrokenCount = "const count: number = 'broken'";
export const createVueRepairedCount = "const count: number = 2";

/** Materialize the clean side shared by check, LSP, and packaged-editor oracles. */
export function materializeCreateVueTypecheckSource(fixture: PinnedFixtureWorkspace): string {
  fixture.applyExactPatch(
    createVueTypecheckAppPath,
    '<script setup lang="ts"></script>',
    `<script setup lang="ts">\n${createVueCleanCount}\nconst label = 'ready'\n</script>`,
  );
  fixture.applyExactPatch(
    createVueTypecheckAppPath,
    'target="_blank" rel="noopener"',
    'target="_blank" rel="noopener noreferrer"',
  );
  return fixture.applyExactPatch(
    createVueTypecheckAppPath,
    "  <h1>You did it!</h1>",
    "  <h1>{{ label }}</h1>\n  <p>{{ count }}</p>",
  );
}
