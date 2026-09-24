import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dataRendererFixtures } from "./renderer-fixtures-data.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { feedbackRendererFixtures } from "./renderer-fixtures-feedback.ts";
import { formInputRendererFixtures } from "./renderer-fixtures-form-inputs.ts";
import { iconRendererFixtures } from "./renderer-fixtures-icon.ts";
import { layoutRendererFixtures } from "./renderer-fixtures-layout.ts";
import { mediaRendererFixtures } from "./renderer-fixtures-media.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";
import { qrCodeRendererFixtures } from "./renderer-fixtures-qr-code.ts";
import { selectionRendererFixtures } from "./renderer-fixtures-selection.ts";

export const groupedRendererFixtures = [
  ...commandRendererFixtures,
  ...dataRendererFixtures,
  ...dialogRendererFixtures,
  ...feedbackRendererFixtures,
  ...formInputRendererFixtures,
  ...iconRendererFixtures,
  ...layoutRendererFixtures,
  ...mediaRendererFixtures,
  ...navigationRendererFixtures,
  ...overlayRendererFixtures,
  ...primitiveRendererFixtures,
  ...qrCodeRendererFixtures,
  ...selectionRendererFixtures,
] as const;
