import { applyIsolatedAliasOverlay } from "./typecheck-baseline-outside-aliases.ts";
import { applyIsolatedJsxOverlay } from "./typecheck-baseline-outside-jsx.ts";
import {
  writeIsolatedTsconfigOverlay,
  type IsolatedOverlay,
} from "./typecheck-baseline-outside-paths.ts";

/**
 * Write every isolated tsconfig overlay Vize and vue-tsc share (#4461).
 * JSX retargeting runs after path/alias rewrite so it can merge into the
 * same untracked overlay without growing `typecheck-dependency-prepare.mjs`.
 */
export function applyIsolatedTypecheckOverlays(
  fixtureRoot: string,
  sourceConfigPath: string,
): IsolatedOverlay | null {
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
