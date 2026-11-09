import type { BunPlugin } from "unplugin";
import { vizeUnplugin } from "./unplugin.ts";
import type { VizeUnpluginOptions } from "./types.ts";

/** Bun plugin factory backed by the shared Vize unplugin implementation. */
const vizeBunPlugin: (options?: VizeUnpluginOptions) => BunPlugin = vizeUnplugin.bun;

export default vizeBunPlugin;
