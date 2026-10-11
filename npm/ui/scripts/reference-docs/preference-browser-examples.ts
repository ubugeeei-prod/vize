/** Complete native connectivity and color preference examples. */
export const preferenceBrowserExamples = [
  {
    name: "network",
    title: "Keep a local guide draft usable when the browser goes offline",
    purpose:
      "Observe actual browser connectivity and optional connection estimates without confusing an online flag with successful service access.",
    observe:
      "Edit Local guide draft, then use the browser's offline switch. Browser status follows the native online/offline event and records its actual event time. The draft and Review local draft remain usable while offline. Reconnect to see the online event. Unknown connection estimates stay explicitly unknown.",
    context:
      "useNetwork uses the actual window/navigator, native online/offline events, and navigator.connection where supported. isSupported describes the optional Network Information API, not whether online/offline observation works. The browser's onLine flag cannot certify service reachability; downlink, class, and round-trip values are estimates. Event timestamps start null and update only on actual events. SSR starts with its default true fallback and null quality measurements, then component mounting reads the browser. Native input and the local review counter belong to the example and send no request or save. Scope disposal removes all window/connection listeners. No network host is injected.",
  },
  {
    name: "color-mode",
    title: "Choose an actual guide color preference or follow the system",
    purpose:
      "Apply actual light, dark, automatic, or configured contrast modes while distinguishing the selected preference, system preference, and resolved mode.",
    observe:
      "Choose Dark or Light with the native select to override system changes. Automatic follows the real browser color-scheme preference. Contrast applies its configured custom mode to the sample card. Follow system preference restores Automatic. When storage is available, reloading reads the previous selected preference after hydration.",
    context:
      "useColorMode observes the actual prefers-color-scheme media query, applies data-guide-mode to this preview's own documentElement, and uses its actual origin's localStorage with a dedicated key. The source-owned CSS styles the sample card from that real attribute. mode is the writable selected preference; state resolves auto to system. stored means the value read at initialization/mount, not an immediately updated save receipt. Persistence is best-effort, so an in-tab mode is never labelled saved. SSR initially renders auto with a light system fallback; the component reads storage after mounting. The example disables transition suppression and supplies no fake storage/media host. Scope disposal removes its watchers/listeners. The parent site's theme is unaffected.",
  },
] as const;
