import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const fileUploadFamilyRoot = "src/families/form/file-upload/";

export const fileUploadFamilyCatalog = [
  {
    canonicalName: "file-upload",
    title: "File Upload",
    packageSubpath: "./file-upload",
    entryFile: `${fileUploadFamilyRoot}file-upload.ts`,
    sourceFiles: [
      `${fileUploadFamilyRoot}file-upload-root.vue`,
      `${fileUploadFamilyRoot}file-upload-dropzone.vue`,
      `${fileUploadFamilyRoot}file-upload-trigger.vue`,
      `${fileUploadFamilyRoot}file-upload-clear.vue`,
      `${fileUploadFamilyRoot}file-upload-item-group.vue`,
      `${fileUploadFamilyRoot}file-upload-item.vue`,
      `${fileUploadFamilyRoot}file-upload-item-preview.vue`,
      `${fileUploadFamilyRoot}file-upload-item-name.vue`,
      `${fileUploadFamilyRoot}file-upload-item-size.vue`,
      `${fileUploadFamilyRoot}file-upload-item-delete.vue`,
      `${fileUploadFamilyRoot}file-upload.ts`,
      `${fileUploadFamilyRoot}file-upload-context.ts`,
      `${fileUploadFamilyRoot}file-upload-transfer.ts`,
      `${fileUploadFamilyRoot}file-upload-types.ts`,
      `${fileUploadFamilyRoot}file-upload-validation.ts`,
    ],
    behaviorContract: `${fileUploadFamilyRoot}file-upload.behavior.md`,
    tests: [
      `${fileUploadFamilyRoot}file-upload.test.ts`,
      `${fileUploadFamilyRoot}file-upload-ssr.test.ts`,
      `${fileUploadFamilyRoot}file-upload-transfer.test.ts`,
      `${fileUploadFamilyRoot}file-upload-validation.test.ts`,
    ],
    typeTests: [`${fileUploadFamilyRoot}file-upload.types.test-d.ts`],
    rendererFixture: "FileUploadConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "FileUpload",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}file-upload-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 5_350,
      maximumCssGzipBytes: 0,
    },
    aliases: ["file upload", "dropzone", "file input", "file picker", "attachments"],
    upstreamCoverage: [
      "HTML input type=file",
      "HTML Drag and Drop API",
      "File and Directory Entries API",
      "Ark UI FileUpload",
      "react-dropzone",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
