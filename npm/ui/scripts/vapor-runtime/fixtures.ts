/**
 * Every runtime conformance fixture, in the order the VDOM suite uses.
 *
 * Imported only after {@link registerSfcHooks} has installed the loader for
 * the active lane, so the `.vue` files behind the fixtures compile for it.
 */
import type { RuntimeFixture } from "../../src/conformance/runtime-conformance-fixtures.ts";

/** Load all runtime fixtures through the active module hooks. */
export async function loadRuntimeFixtures(): Promise<readonly RuntimeFixture[]> {
  const [controls, i18n, overlays] = await Promise.all([
    import("../../src/conformance/runtime-conformance-fixtures.ts"),
    import("../../src/conformance/runtime-conformance-i18n-fixtures.ts"),
    import("../../src/conformance/runtime-conformance-overlay-fixtures.ts"),
  ]);
  return [
    ...controls.controlRuntimeFixtures,
    ...i18n.i18nRuntimeFixtures,
    ...overlays.overlayRuntimeFixtures,
  ];
}
