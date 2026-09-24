import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const qrCodeFamilyRoot = "src/families/media/qr-code/";

export const qrCodeFamilyCatalog = [
  {
    canonicalName: "qr-code",
    title: "QR Code",
    packageSubpath: "./qr-code",
    entryFile: `${qrCodeFamilyRoot}qr-code.ts`,
    sourceFiles: [
      `${qrCodeFamilyRoot}qr-code.vue`,
      `${qrCodeFamilyRoot}qr-code.ts`,
      `${qrCodeFamilyRoot}qr-code-encoder.ts`,
      `${qrCodeFamilyRoot}qr-code-error.ts`,
      `${qrCodeFamilyRoot}qr-code-kanji-table.ts`,
      `${qrCodeFamilyRoot}qr-code-kanji.ts`,
      `${qrCodeFamilyRoot}qr-code-segments.ts`,
      `${qrCodeFamilyRoot}qr-code-svg.ts`,
      `${qrCodeFamilyRoot}qr-code-tables.ts`,
      `${qrCodeFamilyRoot}qr-code-types.ts`,
    ],
    behaviorContract: `${qrCodeFamilyRoot}qr-code.behavior.md`,
    tests: [
      `${qrCodeFamilyRoot}qr-code.test.ts`,
      `${qrCodeFamilyRoot}qr-code-ssr.test.ts`,
      `${qrCodeFamilyRoot}qr-code-encoder.test.ts`,
      `${qrCodeFamilyRoot}qr-code-segments.test.ts`,
      `${qrCodeFamilyRoot}qr-code-svg.test.ts`,
    ],
    typeTests: [`${qrCodeFamilyRoot}qr-code.types.test-d.ts`],
    rendererFixture: "QrCodeConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "QrCode",
      retainedSignature: "VIZE_UI_QR_DATA_TOO_LONG",
      maximumJavaScriptGzipBytes: 6_350,
      maximumCssGzipBytes: 0,
    },
    aliases: ["qr-code", "qr", "qrcode", "2d barcode", "matrix code"],
    upstreamCoverage: [
      "ISO/IEC 18004 QR Code model 2",
      "Project Nayuki QR Code generator",
      "Ark UI QR Code",
      "Mantine QR code examples",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
