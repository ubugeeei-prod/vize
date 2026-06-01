/**
 * useApp - App context composable
 */

import { ref, provide, inject, type InjectionKey, type Ref } from "@vue/runtime-core";

export const APP_KEY: InjectionKey<UseAppReturn> = Symbol("fresco-app");

export interface UseAppReturn {
  /** Terminal width */
  width: Ref<number>;
  /** Terminal height */
  height: Ref<number>;
  /** Whether app is running */
  isRunning: Ref<boolean>;
  /** Exit the app */
  exit: (errorOrResult?: unknown) => void;
  /** Force re-render */
  render: () => void;
  /** Clear the screen */
  clear: () => void;
  /** Wait until pending render output has flushed */
  waitUntilRenderFlush: () => Promise<void>;
}

export interface AppContextControls {
  exit?: (errorOrResult?: unknown) => void;
  render?: () => void;
  clear?: () => void;
  waitUntilRenderFlush?: () => Promise<void>;
  stdout?: NodeJS.WriteStream;
  width?: number;
  height?: number;
}

/**
 * Create app context (use at app root)
 */
export function createAppContext(controls: AppContextControls = {}): UseAppReturn {
  const width = ref(controls.width ?? 80);
  const height = ref(controls.height ?? 24);
  const isRunning = ref(true);

  const exit = (errorOrResult?: unknown) => {
    isRunning.value = false;
    controls.exit?.(errorOrResult);
  };

  const render = () => {
    controls.render?.();
  };

  const clear = () => {
    controls.clear?.();
  };

  const waitUntilRenderFlush = () => {
    return controls.waitUntilRenderFlush?.() ?? Promise.resolve();
  };

  const stdout = controls.stdout ?? (typeof process !== "undefined" ? process.stdout : undefined);

  // Try to get terminal size
  if (stdout) {
    width.value = controls.width ?? stdout.columns ?? 80;
    height.value = controls.height ?? stdout.rows ?? 24;

    stdout.on?.("resize", () => {
      width.value = stdout.columns ?? 80;
      height.value = stdout.rows ?? 24;
    });
  }

  return {
    width,
    height,
    isRunning,
    exit,
    render,
    clear,
    waitUntilRenderFlush,
  };
}

/**
 * Provide app context to descendants
 */
export function provideApp(context: UseAppReturn) {
  provide(APP_KEY, context);
}

/**
 * Use app context
 */
export function useApp(): UseAppReturn {
  const context = inject(APP_KEY);

  if (!context) {
    // Return defaults if not in app context
    return {
      width: ref(80),
      height: ref(24),
      isRunning: ref(false),
      exit: () => {},
      render: () => {},
      clear: () => {},
      waitUntilRenderFlush: () => Promise.resolve(),
    };
  }

  return context;
}

/**
 * Use terminal dimensions
 */
export function useTerminalSize() {
  const { width, height } = useApp();
  return { width, height };
}

/**
 * Exit handler
 */
export function useExit() {
  const { exit } = useApp();
  return exit;
}
