import type { IScriptSnapshot, Language, LanguagePlugin, VirtualCode } from "@volar/language-core";
import type { LanguageProjection } from "@vizejs/native/language-projection.js";
import type { VueLanguageCodeOptions } from "@vizejs/native";
import type * as ts from "typescript";
import type {} from "@volar/typescript";

export interface VizeLanguageOptions extends Omit<VueLanguageCodeOptions, "filename"> {
  /** Opt in to Canon's Vue/Musea variant projection for .art and .art.vue. */
  includeArt?: boolean;
}
export declare class VizeVirtualCode implements VirtualCode {
  id: string;
  languageId: string;
  fileName: string;
  snapshot: IScriptSnapshot;
  source: string;
  projection: LanguageProjection;
  mappings: [];
  embeddedCodes: VirtualCode[];
  constructor(
    typescript: typeof ts,
    fileName: string,
    languageId: string,
    snapshot: IScriptSnapshot,
    options: Omit<VueLanguageCodeOptions, "filename">,
  );
  update(snapshot: IScriptSnapshot): void;
}
export declare function createVizeLanguagePlugin(
  typescript: typeof ts,
  options?: VizeLanguageOptions,
): LanguagePlugin<string, VizeVirtualCode>;
export declare function createVizeLanguageService(
  typescript: typeof ts,
  host: ts.LanguageServiceHost,
  options?: VizeLanguageOptions,
): {
  language: Language<string>;
  languageService: ts.LanguageService;
  plugin: LanguagePlugin<string, VizeVirtualCode>;
};
