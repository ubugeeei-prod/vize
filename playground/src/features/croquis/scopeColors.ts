/** Map scope kind strings to CSS color class names for scope visualization. */
export function getScopeColorClass(kind: string): string {
  // Direct mapping for exact matches
  const classes: Record<string, string> = {
    // Module scope
    mod: "scope-module",
    Mod: "scope-module",
    module: "scope-module",
    Module: "scope-module",
    // Plain (non-script-setup)
    plain: "scope-non-script-setup",
    Plain: "scope-non-script-setup",
    nonScriptSetup: "scope-non-script-setup",
    NonScriptSetup: "scope-non-script-setup",
    // Script setup
    setup: "scope-script-setup",
    Setup: "scope-script-setup",
    scriptSetup: "scope-script-setup",
    ScriptSetup: "scope-script-setup",
    // Function scopes
    function: "scope-function",
    Function: "scope-function",
    arrowFunction: "scope-function",
    ArrowFunction: "scope-function",
    block: "scope-block",
    Block: "scope-block",
    Callback: "scope-callback",
    // Template scopes
    vFor: "scope-vfor",
    VFor: "scope-vfor",
    vSlot: "scope-vslot",
    VSlot: "scope-vslot",
    EventHandler: "scope-event-handler",
    // External modules
    extern: "scope-external-module",
    extmod: "scope-external-module",
    ExternalModule: "scope-external-module",
    // SSR scopes
    universal: "scope-universal",
    Universal: "scope-universal",
    JsGlobal: "scope-js-global-universal",
    client: "scope-client-only",
    Client: "scope-client-only",
    clientOnly: "scope-client-only",
    ClientOnly: "scope-client-only",
    server: "scope-js-global-node",
    Server: "scope-js-global-node",
    // JS Global scopes (runtime-specific)
    jsGlobalUniversal: "scope-js-global-universal",
    JsGlobalUniversal: "scope-js-global-universal",
    jsGlobalBrowser: "scope-js-global-browser",
    JsGlobalBrowser: "scope-js-global-browser",
    jsGlobalNode: "scope-js-global-node",
    JsGlobalNode: "scope-js-global-node",
    jsGlobalDeno: "scope-js-global-deno",
    JsGlobalDeno: "scope-js-global-deno",
    jsGlobalBun: "scope-js-global-bun",
    JsGlobalBun: "scope-js-global-bun",
    // Vue global
    vue: "scope-vue-global",
    Vue: "scope-vue-global",
    vueGlobal: "scope-vue-global",
    VueGlobal: "scope-vue-global",
  };

  // Check for exact match
  if (classes[kind]) {
    return classes[kind];
  }

  // Check for partial matches (e.g., "ClientOnly (onMounted)" should match 'scope-client-only')
  if (kind.startsWith("ClientOnly")) return "scope-client-only";
  if (kind.startsWith("Universal")) return "scope-universal";
  if (kind.startsWith("ServerOnly")) return "scope-js-global-node";
  if (kind.startsWith("Function")) return "scope-function";
  if (kind.startsWith("Arrow")) return "scope-function";
  if (kind.startsWith("ExtMod")) return "scope-external-module";
  if (kind.startsWith("v-for")) return "scope-vfor";
  if (kind.startsWith("v-slot")) return "scope-vslot";
  if (kind.startsWith("@")) return "scope-event-handler"; // Event handlers like @click
  if (kind.includes("computed")) return "scope-function";
  if (kind.includes("watch")) return "scope-function";

  return "scope-default";
}
