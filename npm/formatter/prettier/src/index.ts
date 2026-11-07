import { formatSfc } from "@vizejs/native";
import { createPlugin } from "./plugin.ts";

const plugin = createPlugin(formatSfc);
export const { languages, parsers, printers, options } = plugin;
export type { VizePluginOptions } from "./options.ts";
export type { VizeDocument } from "./plugin.ts";
export default plugin;
