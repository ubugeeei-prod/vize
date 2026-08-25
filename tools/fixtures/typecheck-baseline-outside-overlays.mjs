import { applyIsolatedAliasOverlay } from "./typecheck-baseline-outside-aliases.mjs";
import { applyIsolatedJsxOverlay } from "./typecheck-baseline-outside-jsx.mjs";
import { writeIsolatedTsconfigOverlay } from "./typecheck-baseline-outside-paths.mjs";

/**
 * Write every isolated tsconfig overlay Vize and vue-tsc share (#4461).
 * JSX retargeting runs after path/alias rewrite so it can merge into the
 * same untracked overlay without growing `typecheck-dependency-prepare.mjs`.
 */
export function applyIsolatedTypecheckOverlays(fixtureRoot, sourceConfigPath) {
  return applyIsolatedJsxOverlay(
    fixtureRoot,
    sourceConfigPath,
    applyIsolatedAliasOverlay(
      fixtureRoot,
      sourceConfigPath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourceConfigPath),
    ),
  );
}
