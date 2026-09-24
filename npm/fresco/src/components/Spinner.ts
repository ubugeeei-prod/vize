/**
 * Spinner Component - Loading indicator
 */

import { defineComponent, h, ref, onMounted, onUnmounted, type PropType } from "vue";
import { Text } from "./Text.js";

/**
 * Spinner frame sets
 */
export const spinnerTypes = {
  dots: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
  dots2: ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
  line: ["-", "\\", "|", "/"],
  arc: ["◜", "◠", "◝", "◞", "◡", "◟"],
  circle: ["◐", "◓", "◑", "◒"],
  bounce: ["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
  box: ["▖", "▘", "▝", "▗"],
  arrow: ["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
  clock: ["🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚"],
  moon: ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
  earth: ["🌍", "🌎", "🌏"],
} as const;

export type SpinnerType = keyof typeof spinnerTypes;

export interface SpinnerProps {
  /** Spinner type */
  type?: SpinnerType;
  /** Custom frames */
  frames?: string[];
  /** Animation interval in ms */
  interval?: number;
  /** Label text */
  label?: string;
  /** Foreground color */
  fg?: string;
}

export const Spinner = defineComponent({
  name: "Spinner",
  props: {
    type: {
      type: String as PropType<SpinnerType>,
      default: "dots",
    },
    frames: Array as PropType<string[]>,
    interval: {
      type: Number,
      default: 80,
    },
    label: String,
    fg: String,
  },
  setup(props) {
    const frameIndex = ref(0);
    let timer: ReturnType<typeof setInterval> | null = null;

    const frames = props.frames ?? spinnerTypes[props.type] ?? spinnerTypes.dots;

    onMounted(() => {
      timer = setInterval(() => {
        frameIndex.value = (frameIndex.value + 1) % frames.length;
      }, props.interval);
    });

    onUnmounted(() => {
      if (timer) {
        clearInterval(timer);
      }
    });

    return () => {
      const frame = frames[frameIndex.value];
      const content = props.label ? `${frame} ${props.label}` : frame;

      return h(Text, { fg: props.fg }, () => content);
    };
  },
});
