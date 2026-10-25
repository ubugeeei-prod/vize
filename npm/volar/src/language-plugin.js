import { createRequire } from "node:module";
import {
  LanguageFeature as Feature,
  toUtf16LanguageProjection,
} from "@vizejs/native/language-projection.js";

const require = createRequire(import.meta.url);

export class VizeVirtualCode {
  id = "root";
  mappings = [];
  embeddedCodes = [];

  constructor(ts, fileName, languageId, snapshot, options) {
    this.ts = ts;
    this.fileName = fileName;
    this.languageId = languageId;
    this.options = options;
    this.update(snapshot);
  }

  update(snapshot) {
    this.snapshot = snapshot;
    const source = snapshot.getText(0, snapshot.getLength());
    if (source === this.source) return;
    const native = require("@vizejs/native");
    if (typeof native.generateVueLanguageCode !== "function") {
      throw new Error("@vizejs/volar-plugin requires the matching native language projection API");
    }
    this.projection = toUtf16LanguageProjection(
      source,
      native.generateVueLanguageCode(source, { ...this.options, filename: this.fileName }),
    );
    this.source = source;
    const { projection } = this;
    this.embeddedCodes = [
      {
        id: "typescript",
        languageId: "typescriptreact",
        snapshot: this.ts.ScriptSnapshot.fromString(projection.text),
        mappings: projection.mappings.map(
          ([generated, generatedLength, original, length, kind, features]) => {
            const has = (feature) => (features & feature) === feature;
            const exact =
              kind === 0 &&
              length === generatedLength &&
              source.slice(original, original + length) ===
                projection.text.slice(generated, generated + generatedLength);
            return {
              sourceOffsets: [original],
              generatedOffsets: [generated],
              lengths: [length],
              generatedLengths: [generatedLength],
              data: {
                // A zero feature mask is meaningful: it retains diagnostics only.
                verification: true,
                semantic: has(Feature.Hover)
                  ? { shouldHighlight: () => has(Feature.SemanticTokens) }
                  : false,
                completion: has(Feature.Completion),
                navigation: has(Feature.Definition | Feature.References)
                  ? {
                      shouldHighlight: () => has(Feature.DocumentHighlights),
                      shouldRename: () => exact && has(Feature.Rename),
                    }
                  : false,
                structure: false,
                format: false,
              },
            };
          },
        ),
        linkedCodeMappings: projection.semanticLinks.flatMap((link) => {
          // The protocol currently exports only ref-unwrapping links. Generated
          // aliases with different spelling cannot advertise exact rename edits.
          if (
            link.kind !== "vueSetupTemplateRefUnwrap" ||
            link.sourceLength !== link.targetLength ||
            projection.text.slice(link.sourceStart, link.sourceStart + link.sourceLength) !==
              projection.text.slice(link.targetStart, link.targetStart + link.targetLength)
          )
            return [];
          return [
            {
              sourceOffsets: [link.sourceStart],
              generatedOffsets: [link.targetStart],
              lengths: [link.sourceLength],
              data: undefined,
            },
          ];
        }),
      },
    ];
  }
}

/** A standard Volar 2.4 LanguagePlugin using the caller's authoritative snapshot. */
export function createVizeLanguagePlugin(ts, options = {}) {
  const { includeArt = false, ...codeOptions } = options;
  const languageId = (id) => {
    const name = String(id).toLowerCase();
    if (name.endsWith(".vue")) return "vue";
    if (includeArt && name.endsWith(".art")) return "art";
  };
  return {
    extensions: includeArt ? [".vue", ".art"] : [".vue"],
    getLanguageId: languageId,
    createVirtualCode(id, idOfLanguage, snapshot) {
      const expected = languageId(id);
      if (!expected || expected !== idOfLanguage) return;
      return new VizeVirtualCode(ts, String(id), idOfLanguage, snapshot, codeOptions);
    },
    updateVirtualCode(_id, code, snapshot) {
      code.update(snapshot);
      return code;
    },
    disposeVirtualCode(_id, code) {
      code.embeddedCodes = [];
    },
    typescript: {
      extraFileExtensions: (includeArt ? ["vue", "art"] : ["vue"]).map((extension) => ({
        extension,
        isMixedContent: true,
        scriptKind: ts.ScriptKind.Deferred,
      })),
      getServiceScript(root) {
        if (!(root instanceof VizeVirtualCode)) return;
        return {
          code: root.embeddedCodes[0],
          extension: root.projection.extension,
          scriptKind: ts.ScriptKind.TSX,
          preventLeadingOffset: true,
        };
      },
    },
  };
}
