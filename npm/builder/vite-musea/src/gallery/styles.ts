/**
 * CSS theme variables and style definitions for the Musea gallery.
 *
 * CSS is split into separate .css files and imported as text
 * via tsdown's `?inline` support.
 */

// @ts-expect-error -- CSS imported as text via tsdown `?inline`
import stylesBase from "./styles-base.css?inline";
// @ts-expect-error -- CSS imported as text via tsdown `?inline`
import stylesLayout from "./styles-layout.css?inline";
// @ts-expect-error -- CSS imported as text via tsdown `?inline`
import stylesComponents from "./styles-components.css?inline";

/**
 * Generate the full gallery CSS styles string.
 */
export function generateGalleryStyles(): string {
  return `${stylesBase}\n${stylesLayout}\n${stylesComponents}`;
}
