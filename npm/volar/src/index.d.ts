import type { IScriptSnapshot, Language, LanguagePlugin, VirtualCode } from "@volar/language-core";
import type { LanguageProjection } from "@vizejs/native/language-projection.js";
import type { VueLanguageCodeOptions } from "@vizejs/native";
import type * as ts from "typescript";
import type {} from "@volar/typescript";

/** Options shared by the Vize Volar language plugin and language service. */
export interface VizeLanguageOptions extends Omit<VueLanguageCodeOptions, "filename"> {
  /** Opt in to Canon's Vue/Musea variant projection for .art and .art.vue. @default false */
  includeArt?: boolean;
}
/** Volar virtual-code view backed by one native Vize language projection. */
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
/** Create a Volar language plugin that projects supported Vue-family documents through Vize. */
export declare function createVizeLanguagePlugin(
  typescript: typeof ts,
  options?: VizeLanguageOptions,
): LanguagePlugin<string, VizeVirtualCode>;
/** Create a TypeScript language service whose virtual documents are owned by the Vize plugin. */
export declare function createVizeLanguageService(
  typescript: typeof ts,
  host: ts.LanguageServiceHost,
  options?: VizeLanguageOptions,
): {
  language: Language<string>;
  languageService: ts.LanguageService;
  plugin: LanguagePlugin<string, VizeVirtualCode>;
};
