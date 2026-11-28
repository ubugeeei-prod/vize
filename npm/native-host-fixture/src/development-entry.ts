import type { NativeHostBridge } from "./host-contract.ts";

interface DevelopmentGlobal {
  readonly webkit?: { readonly messageHandlers?: { readonly vizeNative?: NativeHostBridge } };
}

/** Development compiler findings are visible on the native diagnostic surface. */
if (import.meta.hot) {
  import.meta.hot.on("vite:error", (finding: { readonly err?: { readonly message?: string } }) => {
    const host = globalThis as typeof globalThis & DevelopmentGlobal;
    host.webkit?.messageHandlers?.vizeNative?.postMessage({
      format: "vize.native-diagnostic",
      formatVersion: 1,
      code: "native-development-compile-failed",
      message: (finding.err?.message ?? "The native application could not compile.").slice(0, 4096),
    });
  });
}

void import("./entry.ts").catch(() => {
  const host = globalThis as typeof globalThis & DevelopmentGlobal;
  host.webkit?.messageHandlers?.vizeNative?.postMessage({
    format: "vize.native-diagnostic",
    formatVersion: 1,
    code: "native-development-load-failed",
    message: "The native development application could not be loaded.",
  });
});
