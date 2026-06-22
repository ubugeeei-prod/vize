/**
 * Variant auto-generation module.
 * Generates .art.vue files from component prop analysis.
 *
 * JS-based fallback logic (extractPropsSimple, generateMinimalArt,
 * generateArtFileJs, and helpers) is extracted into `fallback.ts`.
 */

import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

import {
  extractPropsSimple,
  finalizeArtOutput,
  generateMinimalArt,
  generateArtFileJs,
} from "./fallback.js";
import {
  fromNativeOutput,
  toNativeAutogenConfig,
  toNativePropDefinitions,
  type NativeAutogenOutput,
  type NativePropDefinition,
} from "./native-shape.js";

/**
 * Autogen configuration options.
 */
export interface AutogenOptions {
  /** Maximum number of variants to generate (default: 20) */
  maxVariants?: number;
  /** Include a "Default" variant with all default values (default: true) */
  includeDefault?: boolean;
  /** Include boolean toggle variants (default: true) */
  includeBooleanToggles?: boolean;
  /** Include enum/union variants (default: true) */
  includeEnumVariants?: boolean;
  /** Include boundary value variants for numbers (default: false) */
  includeBoundaryValues?: boolean;
  /** Include empty string variants for optional strings (default: false) */
  includeEmptyStrings?: boolean;
}

/**
 * Prop definition for variant generation.
 */
export interface PropDefinition {
  name: string;
  propType: string;
  required: boolean;
  defaultValue?: unknown;
}

/**
 * Generated variant.
 */
export interface GeneratedVariant {
  name: string;
  isDefault: boolean;
  props: Record<string, unknown>;
  description?: string;
}

/**
 * Autogen output.
 */
export interface AutogenOutput {
  variants: GeneratedVariant[];
  artFileContent: string;
  componentName: string;
}

// Native binding types
interface NativeAutogen {
  generateVariants?: (
    componentPath: string,
    props: NativePropDefinition[],
    config?: ReturnType<typeof toNativeAutogenConfig>,
  ) => NativeAutogenOutput;
  analyzeSfc?: (
    source: string,
    options?: { filename?: string },
  ) => {
    props: Array<{
      name: string;
      type: string;
      required: boolean;
      defaultValue?: unknown;
      default_value?: unknown;
    }>;
    emits: string[];
  };
}

let native: NativeAutogen | null = null;

function loadNative(): NativeAutogen {
  if (native) return native;
  const require = createRequire(import.meta.url);
  try {
    native = require("@vizejs/native") as NativeAutogen;
    return native;
  } catch (e) {
    throw new Error(
      `Failed to load @vizejs/native. Make sure it's installed and built:\n${String(e)}`,
    );
  }
}

/**
 * Generate .art.vue file for a component.
 *
 * @param componentPath - Path to the Vue component file
 * @param options - Auto-generation options
 * @returns Generated .art.vue content and metadata
 */
export async function generateArtFile(
  componentPath: string,
  options: AutogenOptions = {},
): Promise<AutogenOutput> {
  const absolutePath = path.resolve(componentPath);
  const source = await fs.promises.readFile(absolutePath, "utf-8");

  const binding = loadNative();

  // Analyze component to extract props
  let props: PropDefinition[];
  if (binding.analyzeSfc) {
    const analysis = binding.analyzeSfc(source, { filename: absolutePath });
    props = analysis.props.map((p) => ({
      name: p.name,
      propType: p.type,
      required: p.required,
      defaultValue: p.defaultValue ?? p.default_value,
    }));
  } else {
    // Fallback: simple regex-based prop extraction
    props = extractPropsSimple(source);
  }

  if (props.length === 0) {
    // No props found: generate minimal art file
    const componentName = path.basename(componentPath, ".vue");
    const relPath = `./${path.basename(componentPath)}`;
    return {
      variants: [{ name: "Default", isDefault: true, props: {} }],
      artFileContent: generateMinimalArt(componentName, relPath),
      componentName,
    };
  }

  // Use native variant generation if available
  if (binding.generateVariants) {
    const relPath = `./${path.basename(componentPath)}`;
    const result = binding.generateVariants(
      relPath,
      toNativePropDefinitions(props),
      toNativeAutogenConfig(options),
    );

    return finalizeArtOutput(fromNativeOutput(result), relPath, props);
  }

  // Fallback: JS-based generation
  return generateArtFileJs(componentPath, props, options);
}

/**
 * Write generated .art.vue file to disk.
 */
export async function writeArtFile(
  componentPath: string,
  options: AutogenOptions = {},
  outputPath?: string,
): Promise<string> {
  const output = await generateArtFile(componentPath, options);

  const targetPath = outputPath ?? componentPath.replace(/\.vue$/, ".art.vue");

  await fs.promises.mkdir(path.dirname(targetPath), { recursive: true });
  await fs.promises.writeFile(targetPath, output.artFileContent, "utf-8");

  return targetPath;
}
