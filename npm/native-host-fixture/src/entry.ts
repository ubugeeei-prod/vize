import App from "./App.vue";
import { admitFirstHost } from "./native-config.js";
import { createNativeHostRuntime, type NativeHostRuntime } from "./native-renderer.js";
import type { NativeHostBridge } from "./host-contract.js";

interface NativeGlobal {
  readonly webkit?: { readonly messageHandlers?: { readonly vizeNative?: NativeHostBridge } };
  readonly __VIZE_NATIVE_BOOT__?: { readonly scenario?: unknown };
  __VIZE_NATIVE__?: NativeHostRuntime;
}

const host = globalThis as typeof globalThis & NativeGlobal;
const bridge = host.webkit?.messageHandlers?.vizeNative;
if (bridge === undefined) throw new Error("The native application requires the first host bridge.");
const scenario = host.__VIZE_NATIVE_BOOT__?.scenario ?? "counter";
if (scenario !== "counter" && scenario !== "unsupported-capability") {
  bridge.postMessage({
    format: "vize.native-diagnostic",
    formatVersion: 1,
    code: "unsupported-native-scenario",
    message: "The native development scenario is unsupported.",
  });
} else {
  const admission = admitFirstHost(scenario);
  if (!admission.compatible) {
    const refusal = admission.mismatches[0];
    bridge.postMessage({
      format: "vize.native-diagnostic",
      formatVersion: 1,
      code: refusal?.code ?? "invalid-native-adapter",
      message: `The native host cannot provide ${refusal?.capability ?? "the required application capabilities"}.`,
    });
  } else {
    const runtime = createNativeHostRuntime(bridge);
    host.__VIZE_NATIVE__ = runtime;
    runtime.mount(App);
  }
}
