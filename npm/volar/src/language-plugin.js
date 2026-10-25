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
    let nativeProjection = native.generateVueLanguageCode(source, {
      ...this.options,
      filename: this.fileName,
    });
    if (this.languageId === "art") {
      const projection = JSON.parse(nativeProjection);
      if (!projection.diagnostics.some((diagnostic) => diagnostic.code === 100001)) {
        // Canon already projects native variant bodies. Its SFC scanner and
        // Musea parser also distinguish invalid metadata and an empty gallery.
        const descriptor = native.parseSfc(source, { filename: this.fileName });
        const block = descriptor.customBlocks.find(
          (candidate) => candidate.type === "art" || candidate.type === "variant",
        );
        if (block) {
          let messageText;
          try {
            const art = native.parseArt(source, { filename: this.fileName });
            if (art.variants.length === 0)
              messageText = "Musea Art contains no checkable variant; add a <variant> block.";
          } catch (error) {
            messageText = `Invalid Musea Art: ${error instanceof Error ? error.message : String(error)}`;
          }
          if (messageText)
            projection.diagnostics.push({
              start: block.loc.tagStart,
              length: block.loc.tagEnd - block.loc.tagStart,
              code: 100003,
              messageText,
            });
        }
      }
      nativeProjection = JSON.stringify(projection);
    }
    this.projection = toUtf16LanguageProjection(source, nativeProjection);
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
    if (name.endsWith(".art.vue") || name.endsWith(".art")) return includeArt ? "art" : undefined;
    if (name.endsWith(".vue")) return "vue";
  };
  return {
    extensions: includeArt ? [".art.vue", ".vue", ".art"] : [".vue"],
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
      extraFileExtensions: (includeArt ? ["vue", "art.vue", "art"] : ["vue"]).map((extension) => ({
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
