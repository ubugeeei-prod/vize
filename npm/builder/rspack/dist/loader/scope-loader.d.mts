import { LoaderContext } from "@rspack/core";

//#region src/loader/scope-loader.d.ts
interface VizeScopeLoaderOptions {}
declare function vizeScopeLoader(this: LoaderContext<VizeScopeLoaderOptions>, source: string): void;
//#endregion
export { VizeScopeLoaderOptions, vizeScopeLoader as default };
