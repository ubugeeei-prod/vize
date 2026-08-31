/** Compile-only assertions for the UI source install planner contract. */

import {
  UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION,
  createUiSourceInstallDryRunPlan,
  type UiSourceInstallActionOperation,
  type UiSourceInstallMode,
  type UiSourceInstallPlan,
} from "./source-install-planner.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const output = createUiSourceInstallDryRunPlan({
  mode: "dry-run",
  requestedFamilies: ["button"],
  destinationRoot: "src/components",
  sourceFiles: [
    {
      familyName: "button",
      sourcePath: "src/families/actions/button/button.ts",
      destinationPath: "button.ts",
      sourceDigest: "sha256:button",
    },
  ],
});

type _PlannerModeIsDryRunOnly = Expect<Equal<UiSourceInstallMode, "dry-run">>;
type _PlanSchemaVersionIsLiteral = Expect<
  Equal<typeof output.schemaVersion, typeof UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION>
>;
type _PlanShapeIsClosed = Expect<Equal<typeof output, UiSourceInstallPlan>>;
type _OperationsAreClosed = Expect<
  Equal<UiSourceInstallActionOperation, "conflict" | "create" | "overwrite" | "skip">
>;

createUiSourceInstallDryRunPlan({
  // @ts-expect-error dry-run is the only supported planner mode in this slice.
  mode: "write",
  requestedFamilies: ["button"],
  destinationRoot: "src/components",
  sourceFiles: [],
});

// @ts-expect-error plan actions are immutable to callers.
output.actions.push(output.actions[0]);
// @ts-expect-error planner output cannot be marked successful by consumers.
output.ok = true;
