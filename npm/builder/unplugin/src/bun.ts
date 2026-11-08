import type { BunPlugin } from "unplugin";
import { vizeUnplugin } from "./unplugin.ts";
import type { VizeUnpluginOptions } from "./types.ts";

const vizeBunPlugin: (options?: VizeUnpluginOptions) => BunPlugin = vizeUnplugin.bun;

export default vizeBunPlugin;
