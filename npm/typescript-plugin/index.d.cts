import type * as ts from "typescript/lib/tsserverlibrary";
declare function init(modules: { typescript: typeof ts }): ts.server.PluginModule;
export = init;
