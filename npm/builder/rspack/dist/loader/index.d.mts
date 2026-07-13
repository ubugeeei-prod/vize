import { d as VizeLoaderOptions } from "../index-sdINSvBH.mjs";
import { LoaderContext } from "@rspack/core";

//#region src/loader/index.d.ts
declare function vizeLoader(this: LoaderContext<VizeLoaderOptions>, source: string): void;
//#endregion
export { vizeLoader as default };
