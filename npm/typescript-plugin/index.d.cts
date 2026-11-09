import type * as ts from "typescript/lib/tsserverlibrary";
/** Create the Vize TypeScript server plugin for the host TypeScript instance. */
declare function init(modules: { typescript: typeof ts }): ts.server.PluginModule;
export = init;
