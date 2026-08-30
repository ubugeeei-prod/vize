import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { iconRendererFixtures } from "./renderer-fixtures-icon.ts";
import { layoutRendererFixtures } from "./renderer-fixtures-layout.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";
import { selectionRendererFixtures } from "./renderer-fixtures-selection.ts";
import { statusLightRendererFixtures } from "./renderer-fixtures-status-light.ts";

export const groupedRendererFixtures = [
  ...commandRendererFixtures,
  ...dialogRendererFixtures,
  ...iconRendererFixtures,
  ...layoutRendererFixtures,
  ...navigationRendererFixtures,
  ...overlayRendererFixtures,
  ...primitiveRendererFixtures,
  ...selectionRendererFixtures,
  ...statusLightRendererFixtures,
] as const;
