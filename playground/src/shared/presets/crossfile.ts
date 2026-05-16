/**
 * CrossFile Playground Presets
 *
 * A comprehensive set of presets demonstrating cross-file analysis patterns:
 *
 * - default: General provide/inject and emit patterns
 * - reactivity: Reactivity loss patterns (destructuring, spreading)
 * - modules: Module-level state and CSRP risks
 * - reference-escape: Reference escape patterns
 * - provide-inject: Provide/inject type safety
 * - attrs: $attrs handling patterns
 *
 * Note: This file is separate from the Vue component to avoid
 * linting issues with embedded Vue code in template literals.
 */

import {
  mdiDiamond,
  mdiFlash,
  mdiAlert,
  mdiArrowTopRight,
  mdiFileTree,
  mdiArrowDown,
} from "@mdi/js";

import { OVERVIEW_PRESET } from "./crossfile-overview";
import { REACTIVITY_PRESET } from "./crossfile-reactivity";
import { SETUP_CONTEXT_PRESET } from "./crossfile-setup-context";
import { REFERENCE_ESCAPE_PRESET } from "./crossfile-reference-escape";
import { PROVIDE_INJECT_PRESET } from "./crossfile-provide-inject";
import { FALLTHROUGH_ATTRS_PRESET } from "./crossfile-attrs";

export interface Preset {
  id: string;
  name: string;
  description: string;
  icon: string;
  files: Record<string, string>;
}

export const PRESETS: Preset[] = [
  { ...OVERVIEW_PRESET, icon: mdiDiamond },
  { ...REACTIVITY_PRESET, icon: mdiFlash },
  { ...SETUP_CONTEXT_PRESET, icon: mdiAlert },
  { ...REFERENCE_ESCAPE_PRESET, icon: mdiArrowTopRight },
  { ...PROVIDE_INJECT_PRESET, icon: mdiFileTree },
  { ...FALLTHROUGH_ATTRS_PRESET, icon: mdiArrowDown },
];
