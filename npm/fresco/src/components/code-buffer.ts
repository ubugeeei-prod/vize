import { ref } from "vue";
import type { CodeBuffer } from "./code-types.js";

function normalizeLimit(maxLines: number | undefined): number | undefined {
  if (maxLines === undefined || !Number.isFinite(maxLines)) return undefined;
  return Math.max(1, Math.trunc(maxLines));
}

function retainLastLines(content: string, maxLines: number | undefined): string {
  const limit = normalizeLimit(maxLines);
  if (limit === undefined) return content;
  const lines = content.split("\n");
  return lines.length <= limit ? content : lines.slice(-limit).join("\n");
}

/** Create a reactive stream buffer with optional bounded line retention. */
export function createCodeBuffer(initial = "", options: { maxLines?: number } = {}): CodeBuffer {
  const content = ref(retainLastLines(initial, options.maxLines));
  return {
    content,
    append: (chunk) => {
      content.value = retainLastLines(`${content.value}${chunk}`, options.maxLines);
    },
    replace: (next) => {
      content.value = retainLastLines(next, options.maxLines);
    },
    clear: () => {
      content.value = "";
    },
  };
}
