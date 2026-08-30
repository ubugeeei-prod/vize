import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";
import { selectionRendererFixtures } from "./renderer-fixtures-selection.ts";
import { statusLightRendererFixtures } from "./renderer-fixtures-status-light.ts";

export const groupedRendererFixtures = [
  ...commandRendererFixtures,
  ...dialogRendererFixtures,
  ...navigationRendererFixtures,
  ...overlayRendererFixtures,
  ...primitiveRendererFixtures,
  ...selectionRendererFixtures,
  ...statusLightRendererFixtures,
] as const;
