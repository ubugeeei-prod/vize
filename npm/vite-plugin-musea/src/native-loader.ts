/**
 * Native binding loader for @vizejs/native.
 *
 * Provides lazy-loading of the native Rust-based parser and a JS fallback
 * for SFC analysis when the native `analyzeSfc` function is unavailable.
 */

import { createRequire } from "node:module";

// Native binding types
export interface NativeBinding {
  parseArt: (
    source: string,
    options?: { filename?: string },
  ) => {
    filename: string;
    metadata: {
      title: string;
      description?: string;
      component?: string;
      category?: string;
      tags: string[];
      status: string;
      order?: number;
      actionEvents?: string[];
    };
    variants: Array<{
      name: string;
      template: string;
      isDefault: boolean;
      skipVrt: boolean;
    }>;
    hasScriptSetup: boolean;
    hasScript: boolean;
    styleCount: number;
  };
  artToCsf: (
    source: string,
    options?: { filename?: string },
  ) => {
    code: string;
    filename: string;
  };
  generateArtPalette?: (
    source: string,
    artOptions?: { filename?: string },
    paletteOptions?: { infer_options?: boolean; group_by_type?: boolean },
  ) => {
    title: string;
    controls: Array<{
      name: string;
      control: string;
      default_value?: unknown;
      description?: string;
      required: boolean;
      options: Array<{ label: string; value: unknown }>;
      range?: { min: number; max: number; step?: number };
      group?: string;
    }>;
    groups: string[];
    json: string;
    typescript: string;
  };
  generateArtDoc?: (
    source: string,
    artOptions?: { filename?: string },
    docOptions?: {
      include_source?: boolean;
      include_templates?: boolean;
      include_metadata?: boolean;
    },
  ) => {
    markdown: string;
    filename: string;
    title: string;
    category?: string;
    variant_count: number;
  };
  analyzeSfc?: (
    source: string,
    options?: { filename?: string },
  ) => {
    props: Array<{
      name: string;
      type: string;
      required: boolean;
      default_value?: unknown;
    }>;
    emits: string[];
  };
}

// Lazy-load native binding
let native: NativeBinding | null = null;

export function loadNative(): NativeBinding {
  if (native) return native;

  const require = createRequire(import.meta.url);
  try {
    native = require("@vizejs/native") as NativeBinding;
    return native;
  } catch (e) {
    throw new Error(
      `Failed to load @vizejs/native. Make sure it's installed and built:\n${String(e)}`,
    );
  }
}

/**
 * JS-based fallback for SFC analysis when native `analyzeSfc` is not available.
 * Uses regex parsing to extract props and emits from Vue SFC source.
 */
export function analyzeSfcFallback(
  source: string,
  _options?: { filename?: string },
): {
  props: Array<{
    name: string;
    type: string;
    required: boolean;
    default_value?: unknown;
  }>;
  emits: string[];
} {
  try {
    const props: Array<{
      name: string;
      type: string;
      required: boolean;
      default_value?: unknown;
    }> = [];
    const emits: string[] = [];

    // Extract the <script setup> block
    const scriptSetupMatch = source.match(/<script\s+[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
    if (!scriptSetupMatch) {
      // Try regular <script> block
      const scriptMatch = source.match(/<script[^>]*>([\s\S]*?)<\/script>/);
      if (!scriptMatch) return { props: [], emits: [] };
    }
    const scriptContent = scriptSetupMatch?.[1] || "";

    // Extract defineProps type parameter
    // Handles: defineProps<{ ... }>()  and  defineProps<{ ... }>
    const propsMatch = scriptContent.match(/defineProps\s*<\s*\{([\s\S]*?)\}>\s*\(/);
    const propsMatch2 = scriptContent.match(/defineProps\s*<\s*\{([\s\S]*?)\}>/);
    const propsBody = propsMatch?.[1] || propsMatch2?.[1];

    if (propsBody) {
      // Parse each prop line: name?: Type;  or  name: Type;
      // Handle multiline JSDoc comments before props
      const lines = propsBody.split("\n");
      let i = 0;
      while (i < lines.length) {
        const line = lines[i].trim();
        // Skip JSDoc comments
        if (line.startsWith("/**") || line.startsWith("*") || line.startsWith("*/")) {
          i++;
          continue;
        }

        // Match prop definition: name?: Type  or  name: Type
        const propMatch = line.match(/^(\w+)(\?)?:\s*(.+?)(?:;?\s*)$/);
        if (propMatch) {
          const name = propMatch[1];
          const optional = !!propMatch[2];
          let type = propMatch[3].replace(/;$/, "").trim();

          // Check for default value in destructured defineProps
          const defaultPattern = new RegExp(`\\b${name}\\s*=\\s*([^,}\\n]+)`);
          const defaultMatch = scriptContent.match(defaultPattern);
          const defaultValue = defaultMatch ? defaultMatch[1].trim() : undefined;

          props.push({
            name,
            type,
            required: !optional && defaultValue === undefined,
            ...(defaultValue !== undefined ? { default_value: defaultValue } : {}),
          });
        }
        i++;
      }
    }

    // Extract defineEmits
    const emitsMatch = scriptContent.match(/defineEmits\s*<\s*\{([\s\S]*?)\}>/);
    if (emitsMatch) {
      const emitsBody = emitsMatch[1];
      const emitRegex = /(\w+)\s*:/g;
      let match;
      while ((match = emitRegex.exec(emitsBody)) !== null) {
        emits.push(match[1]);
      }
    }

    return { props, emits };
  } catch {
    return { props: [], emits: [] };
  }
}
