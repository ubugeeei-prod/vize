import {
  i as VizeVueVersion,
  n as VizeCompatibilityOptions,
  r as VizeUnpluginOptions,
  t as MacroArtifact,
} from "./types-D3or3aSV.cjs";
import * as _$unplugin from "unplugin";
import { Compiler } from "webpack";

//#region src/unplugin.d.ts
declare const vizeUnplugin: _$unplugin.UnpluginInstance<VizeUnpluginOptions | undefined, boolean>;
//#endregion
export {
  type MacroArtifact,
  type VizeCompatibilityOptions,
  type VizeUnpluginOptions,
  type VizeVueVersion,
  vizeUnplugin as default,
  vizeUnplugin,
};
