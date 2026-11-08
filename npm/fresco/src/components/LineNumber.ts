/**
 * Standalone line-number gutter for code, diagnostics, and diff views.
 */

import { computed, defineComponent, h } from "vue";

export interface LineNumberProps {
  /** Line number displayed in the gutter. */
  value: number;
  /** Minimum character width reserved for the number. */
  width?: number;
  /** Text placed between the number and its content. */
  separator?: string;
  /** Spaces rendered on each side of the separator. */
  padding?: number;
  /** Default foreground color. */
  fg?: string;
  /** Whether this line is the active line. */
  active?: boolean;
  /** Foreground color used for the active line. */
  activeFg?: string;
  /** Background color used for the active line. */
  activeBg?: string;
}

function normalizeInteger(value: number, minimum: number): number {
  if (!Number.isFinite(value)) return minimum;
  return Math.max(minimum, Math.trunc(value));
}

export const LineNumber = defineComponent({
  name: "LineNumber",
  props: {
    value: {
      type: Number,
      required: true,
    },
    width: {
      type: Number,
      default: 0,
    },
    separator: {
      type: String,
      default: "│",
    },
    padding: {
      type: Number,
      default: 1,
    },
    fg: {
      type: String,
      default: "gray",
    },
    active: Boolean,
    activeFg: {
      type: String,
      default: "white",
    },
    activeBg: String,
  },
  setup(props) {
    const line = computed(() => (Number.isFinite(props.value) ? Math.trunc(props.value) : 0));
    const text = computed(() => {
      const width = Math.max(normalizeInteger(props.width, 0), String(line.value).length);
      const padding = " ".repeat(normalizeInteger(props.padding, 0));
      return `${String(line.value).padStart(width)}${padding}${props.separator}${padding}`;
    });

    return () =>
      h("text", {
        text: text.value,
        fg: props.active ? props.activeFg : props.fg,
        bg: props.active ? props.activeBg : undefined,
        bold: props.active,
        "aria-label": `Line ${line.value}`,
      });
  },
});
