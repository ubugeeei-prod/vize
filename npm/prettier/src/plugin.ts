import type { FormatOptionsNapi, FormatResultNapi } from "@vizejs/native";
import { doc, type Plugin } from "prettier";
import { assertSupportedDirectives, hasIgnorePragma, hasPragma } from "./directives.ts";
import { nativeOptions, type VizeOptions } from "./options.ts";

export interface VizeDocument {
  type: "VizeDocument";
  code: string;
  sourceLength: number;
  hadPragma: boolean;
}

type NativeFormat = (source: string, options: FormatOptionsNapi) => FormatResultNapi;

export function createPlugin(format: NativeFormat): Plugin<VizeDocument> {
  const parser = {
    astFormat: "vize-document",
    async parse(text: string, options: VizeOptions): Promise<VizeDocument> {
      const mapped = nativeOptions(options, text.length);
      await assertSupportedDirectives(text, options);
      const { code } = format(text, mapped);
      // Location metadata must not participate in the host's debug AST equality.
      return Object.defineProperties(
        { type: "VizeDocument" as const, code },
        {
          sourceLength: { value: text.length },
          hadPragma: { value: hasPragma(text) },
        },
      ) as VizeDocument;
    },
    locStart: () => 0,
    locEnd: (node: VizeDocument) => node.sourceLength,
    hasPragma,
    hasIgnorePragma,
  };
  return {
    languages: [
      { name: "Vue", parsers: ["vize"], extensions: [".vue"], vscodeLanguageIds: ["vue"] },
      { name: "Art", parsers: ["vize-art"], extensions: [".art"] },
    ],
    parsers: { vize: parser, "vize-art": parser, vue: parser },
    printers: {
      "vize-document": {
        print: (path, options) => {
          const body = doc.builders.join(doc.builders.literalline, path.node.code.split("\n"));
          return options.insertPragma && !path.node.hadPragma
            ? ["<!-- @format -->", doc.builders.literalline, doc.builders.literalline, body]
            : body;
        },
        getVisitorKeys: () => [],
        // Emit the pragma in the Doc so input ranges and cursor geometry stay valid.
        insertPragma: (text) => text,
      },
    },
    options: {
      vizeVueVersion: {
        type: "choice",
        category: "Vize",
        default: "3",
        description: "Vue template grammar used by the shared formatter.",
        choices: [
          { value: "2", description: "Vue 2" },
          { value: "3", description: "Vue 3" },
        ],
      },
    },
  };
}
