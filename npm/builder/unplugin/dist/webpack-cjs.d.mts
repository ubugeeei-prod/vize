import { r as VizeUnpluginOptions } from "./types-DmgrThqW.mjs";
import { Compiler } from "webpack";

//#region src/webpack-cjs.d.ts
type WebpackPlugin = {
  apply(compiler: Compiler): void;
};
declare function vizeWebpackCjs(options?: VizeUnpluginOptions): WebpackPlugin;
//#endregion
export { vizeWebpackCjs as default, vizeWebpackCjs };
