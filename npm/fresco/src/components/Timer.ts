/**
 * Timer Component - Countdown/stopwatch timer
 */

import { defineComponent, h, ref, onMounted, onUnmounted, type PropType, computed } from "vue";

export type TimerMode = "countdown" | "stopwatch";

export interface TimerProps {
  /** Timer mode */
  mode?: TimerMode;
  /** Initial seconds (for countdown) */
  initialSeconds?: number;
  /** Auto start */
  autoStart?: boolean;
  /** Show hours */
  showHours?: boolean;
  /** Show milliseconds */
  showMilliseconds?: boolean;
  /** Foreground color */
  fg?: string;
  /** Warning color (when < 10 seconds in countdown) */
  warningFg?: string;
  /** Danger color (when < 5 seconds in countdown) */
  dangerFg?: string;
}

export const Timer = defineComponent({
  name: "Timer",
  props: {
    mode: {
      type: String as PropType<TimerMode>,
      default: "stopwatch",
    },
    initialSeconds: {
      type: Number,
      default: 0,
    },
    autoStart: {
      type: Boolean,
      default: true,
    },
    showHours: {
      type: Boolean,
      default: false,
    },
    showMilliseconds: {
      type: Boolean,
      default: false,
    },
    fg: {
      type: String,
      default: "white",
    },
    warningFg: {
      type: String,
      default: "yellow",
    },
    dangerFg: {
      type: String,
      default: "red",
    },
  },
  emits: ["tick", "complete"],
  setup(props, { emit, expose }) {
    const elapsed = ref(0); // milliseconds
    const isRunning = ref(false);
    let intervalId: ReturnType<typeof setInterval> | null = null;

    const totalMs = computed(() => {
      if (props.mode === "countdown") {
        return Math.max(0, props.initialSeconds * 1000 - elapsed.value);
      }
      return elapsed.value;
    });

    const formatted = computed(() => {
      const ms = totalMs.value;
      const totalSeconds = Math.floor(ms / 1000);
      const hours = Math.floor(totalSeconds / 3600);
      const minutes = Math.floor((totalSeconds % 3600) / 60);
      const seconds = totalSeconds % 60;
      const milliseconds = Math.floor((ms % 1000) / 10);

      let result = "";
      if (props.showHours || hours > 0) {
        result += `${String(hours).padStart(2, "0")}:`;
      }
      result += `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
      if (props.showMilliseconds) {
        result += `.${String(milliseconds).padStart(2, "0")}`;
      }
      return result;
    });

    const color = computed(() => {
      if (props.mode === "countdown") {
        const seconds = totalMs.value / 1000;
        if (seconds <= 5) return props.dangerFg;
        if (seconds <= 10) return props.warningFg;
      }
      return props.fg;
    });

    const start = () => {
      if (isRunning.value) return;
      isRunning.value = true;
      intervalId = setInterval(() => {
        elapsed.value += 100;
        emit("tick", totalMs.value);

        if (props.mode === "countdown" && totalMs.value <= 0) {
          stop();
          emit("complete");
        }
      }, 100);
    };

    const stop = () => {
      isRunning.value = false;
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    };

    const reset = () => {
      elapsed.value = 0;
    };

    const toggle = () => {
      if (isRunning.value) {
        stop();
      } else {
        start();
      }
    };

    expose({ start, stop, reset, toggle, isRunning });

    onMounted(() => {
      if (props.autoStart) {
        start();
      }
    });

    onUnmounted(() => {
      stop();
    });

    return () => {
      return h(
        "text",
        {
          fg: color.value,
          bold: true,
        },
        formatted.value,
      );
    };
  },
});
