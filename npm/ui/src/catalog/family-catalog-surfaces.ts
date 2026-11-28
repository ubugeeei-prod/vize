import { chatFamilyCatalogEntry } from "../components/chat/metadata.ts";
import { colorWheelFamilyCatalogEntry } from "../components/color-wheel/metadata.ts";
import { descriptionsFamilyCatalogEntry } from "../components/descriptions/metadata.ts";
import { diffViewFamilyCatalogEntry } from "../components/diff-view/metadata.ts";
import { inputGroupFamilyCatalogEntry } from "../components/input-group/metadata.ts";
import { resultFamilyCatalogEntry } from "../components/result/metadata.ts";
import { segmentedFamilyCatalogEntry } from "../components/segmented/metadata.ts";
import { statisticFamilyCatalogEntry } from "../components/statistic/metadata.ts";
import { watermarkFamilyCatalogEntry } from "../components/watermark/metadata.ts";
import type { UiFamilyCatalogEntry } from "./family-catalog-types.ts";

/** Component units that cover grouped fields, hue, chat, and reading surfaces. */
export const surfaceFamilyCatalog = [
  chatFamilyCatalogEntry,
  colorWheelFamilyCatalogEntry,
  descriptionsFamilyCatalogEntry,
  diffViewFamilyCatalogEntry,
  inputGroupFamilyCatalogEntry,
  resultFamilyCatalogEntry,
  segmentedFamilyCatalogEntry,
  statisticFamilyCatalogEntry,
  watermarkFamilyCatalogEntry,
] as const satisfies readonly UiFamilyCatalogEntry[];
