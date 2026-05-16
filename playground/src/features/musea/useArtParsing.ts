import { ref, computed, watch, type Ref } from "vue";
import type { WasmModule, ArtDescriptor, CsfOutput } from "../../wasm/index";

export interface Diagnostic {
  message: string;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: "error" | "warning" | "info";
}

export interface DesignToken {
  name: string;
  value: string;
  type: "color" | "size" | "other";
}

export function isColorValue(value: string): boolean {
  return /^(#[0-9a-fA-F]{3,8}|rgb|rgba|hsl|hsla|transparent|currentColor|inherit)/i.test(value);
}

export function isSizeValue(value: string): boolean {
  return /^-?\d+(\.\d+)?(px|rem|em|%|vh|vw|vmin|vmax|ch|ex)$/.test(value);
}

export function useArtParsing(
  source: Ref<string>,
  compiler: Ref<WasmModule | null> | (() => WasmModule | null),
) {
  const parsedArt = ref<ArtDescriptor | null>(null);
  const csfOutput = ref<CsfOutput | null>(null);
  const error = ref<string | null>(null);
  const diagnostics = ref<Diagnostic[]>([]);
  const compileTime = ref<number | null>(null);

  const variantCount = computed(() => parsedArt.value?.variants.length ?? 0);

  // Design tokens extraction
  const designTokens = computed((): DesignToken[] => {
    if (!parsedArt.value) return [];

    const tokens: DesignToken[] = [];
    const cssVarRegex = /--([a-zA-Z0-9-]+)\s*:\s*([^;]+)/g;

    // Try to extract from styles array first
    const styles = parsedArt.value.styles || [];
    let styleContent = "";

    if (styles.length > 0) {
      // Use styles from parsed result
      for (const style of styles) {
        styleContent += (style.content || "") + "\n";
      }
    } else {
      // Fallback: extract style content directly from source
      const styleRegex = new RegExp("<style[^>]*>([\\s\\S]*?)</style>", "g");
      let styleMatch;
      while ((styleMatch = styleRegex.exec(source.value)) !== null) {
        styleContent += styleMatch[1] + "\n";
      }
    }

    // Extract CSS variables from style content
    let match;
    while ((match = cssVarRegex.exec(styleContent)) !== null) {
      const name = `--${match[1]}`;
      const value = match[2].trim();
      tokens.push({
        name,
        value,
        type: isColorValue(value) ? "color" : isSizeValue(value) ? "size" : "other",
      });
    }

    return tokens;
  });

  const colorTokens = computed(() => designTokens.value.filter((t) => t.type === "color"));
  const sizeTokens = computed(() => designTokens.value.filter((t) => t.type === "size"));
  const otherTokens = computed(() => designTokens.value.filter((t) => t.type === "other"));

  function getCompiler(): WasmModule | null {
    return typeof compiler === "function" ? compiler() : compiler.value;
  }

  async function compile() {
    const comp = getCompiler();
    if (!comp) return;

    const startTime = performance.now();
    error.value = null;
    diagnostics.value = [];

    try {
      // Parse Art file
      const parsed = comp.parseArt(source.value, {
        filename: "example.art.vue",
      });
      parsedArt.value = parsed;

      // Transform to CSF
      const csf = comp.artToCsf(source.value, {
        filename: "example.art.vue",
      });
      csfOutput.value = csf;

      compileTime.value = performance.now() - startTime;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      error.value = message;
      parsedArt.value = null;
      csfOutput.value = null;

      // Parse line info from error message if available
      const lineMatch = message.match(/line\s*(\d+)/i);
      const colMatch = message.match(/col(?:umn)?\s*(\d+)/i);
      const line = lineMatch ? parseInt(lineMatch[1], 10) : 1;
      const col = colMatch ? parseInt(colMatch[1], 10) : 1;

      diagnostics.value = [
        {
          message,
          startLine: line,
          startColumn: col,
          severity: "error",
        },
      ];
    }
  }

  // Debounced compile on source change
  let compileTimer: ReturnType<typeof setTimeout> | null = null;

  watch(
    source,
    () => {
      if (compileTimer) clearTimeout(compileTimer);
      compileTimer = setTimeout(compile, 300);
    },
    { immediate: true },
  );

  // Re-compile when compiler becomes available
  watch(
    () => getCompiler(),
    () => {
      if (getCompiler()) void compile();
    },
  );

  return {
    parsedArt,
    csfOutput,
    error,
    diagnostics,
    compileTime,
    designTokens,
    colorTokens,
    sizeTokens,
    otherTokens,
    variantCount,
    compile,
  };
}
