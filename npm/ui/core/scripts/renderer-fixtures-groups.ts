import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";

export const groupedRendererFixtures = [
  ...commandRendererFixtures,
  ...dialogRendererFixtures,
  ...navigationRendererFixtures,
  ...overlayRendererFixtures,
  ...primitiveRendererFixtures,
] as const;
