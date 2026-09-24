/** Compile-only assertions for the public FileUpload contract. */

import {
  FileUpload,
  FileUploadClear,
  FileUploadDropzone,
  FileUploadItem,
  FileUploadItemDelete,
  FileUploadItemGroup,
  FileUploadItemName,
  FileUploadItemPreview,
  FileUploadItemSize,
  FileUploadRoot,
  FileUploadTrigger,
  formatFileSize,
  matchAcceptType,
  parseAccept,
  validateFiles,
  type FileUploadAcceptMatch,
  type FileUploadAcceptToken,
  type FileUploadActionExpose,
  type FileUploadAddResult,
  type FileUploadBuiltInRejectionCode,
  type FileUploadDropzoneExpose,
  type FileUploadDropzoneState,
  type FileUploadError,
  type FileUploadItemGroupSlotState,
  type FileUploadItemPreviewSlotState,
  type FileUploadItemSlotState,
  type FileUploadMessages,
  type FileUploadPasteScope,
  type FileUploadRejectDragEffect,
  type FileUploadPreviewState,
  type FileUploadRejection,
  type FileUploadRejectionCode,
  type FileUploadRootEmits,
  type FileUploadRootExpose,
  type FileUploadRootProps,
  type FileUploadSizeStandard,
  type FileUploadSlotState,
  type FileUploadSource,
  type FileUploadState,
  type FileUploadValidator,
} from "./file-upload.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: FileUploadRootExpose;
declare const dropzone: FileUploadDropzoneExpose;
declare const action: FileUploadActionExpose;
declare const slot: FileUploadSlotState;
declare const item: FileUploadItemSlotState;
declare const group: FileUploadItemGroupSlotState;
declare const preview: FileUploadItemPreviewSlotState;
declare const rejection: FileUploadRejection;

type _RootIsAlias = Expect<Equal<typeof FileUpload, typeof FileUploadRoot>>;
type _CodesAreClosed = Expect<
  Equal<
    FileUploadRejectionCode,
    "custom" | "file-invalid-type" | "file-too-large" | "file-too-small" | "too-many-files"
  >
>;
type _BuiltInCodesExcludeCustom = Expect<
  Equal<FileUploadBuiltInRejectionCode, Exclude<FileUploadRejectionCode, "custom">>
>;
type _SourceIsClosed = Expect<Equal<FileUploadSource, "api" | "drop" | "input" | "paste">>;
type _StateIsClosed = Expect<Equal<FileUploadState, "disabled" | "empty" | "filled">>;
type _DropzoneStateIsClosed = Expect<
  Equal<FileUploadDropzoneState, "disabled" | "dragging" | "idle" | "rejecting">
>;
type _PreviewStateIsClosed = Expect<
  Equal<FileUploadPreviewState, "pending" | "ready" | "unsupported">
>;
type _PasteScopeIsClosed = Expect<Equal<FileUploadPasteScope, "document" | "none" | "self">>;
type _RejectDragEffectIsClosed = Expect<Equal<FileUploadRejectDragEffect, "copy" | "none">>;
type _SizeStandardIsClosed = Expect<Equal<FileUploadSizeStandard, "iec" | "si">>;
type _AcceptMatchIsClosed = Expect<Equal<FileUploadAcceptMatch, "accept" | "reject" | "unknown">>;
type _RootFilesAreReadonly = Expect<Equal<typeof root.files, readonly File[]>>;
type _RootElementIsDiv = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _RootInputIsInput = Expect<Equal<typeof root.inputElement, HTMLInputElement | null>>;
type _AddFilesReturnsResult = Expect<Equal<ReturnType<typeof root.addFiles>, FileUploadAddResult>>;
type _RelativePathIsNullable = Expect<
  Equal<ReturnType<typeof root.getRelativePath>, string | null>
>;
type _DropzoneElementIsDiv = Expect<Equal<typeof dropzone.element, HTMLDivElement | null>>;
type _ActionElementIsButton = Expect<Equal<typeof action.element, HTMLButtonElement | null>>;
type _SlotFilesAreReadonly = Expect<Equal<typeof slot.files, readonly File[]>>;
type _ItemPreviewUrlIsNullable = Expect<Equal<typeof item.previewUrl, string | null>>;
type _ItemFileIsFile = Expect<Equal<typeof item.file, File>>;
type _GroupItemsAreItemState = Expect<
  Equal<typeof group.items, readonly FileUploadItemSlotState[]>
>;
type _PreviewExtendsItem = Expect<Equal<typeof preview.state, FileUploadPreviewState>>;
type _RejectionErrorsAreTyped = Expect<Equal<typeof rejection.errors, readonly FileUploadError[]>>;
type _ModelIsReadonlyFiles = Expect<
  Equal<FileUploadRootProps["modelValue"], readonly File[] | undefined>
>;
type _UpdatePayload = Expect<
  Equal<FileUploadRootEmits["update:modelValue"], [files: readonly File[]]>
>;
type _RejectPayload = Expect<
  Equal<
    FileUploadRootEmits["reject"],
    [rejections: readonly FileUploadRejection[], source: FileUploadSource]
  >
>;
type _AcceptTokensAreDiscriminated = Expect<
  Equal<FileUploadAcceptToken["kind"], "any" | "extension" | "mime">
>;

root.openPicker();
root.addFiles([new File([], "a.txt")]);
root.addFiles(new Set<File>());
dropzone.focus({ preventScroll: true });
action.focus();

const validator: FileUploadValidator = (file, accepted) =>
  file.size > accepted.length ? "file-too-large" : null;
const objectValidator: FileUploadValidator = () => ({ code: "custom", message: "No" });
const messageValidator: FileUploadValidator = () => "Any custom message";
const messages: FileUploadMessages = {
  "file-too-large": ({ formatSize, maxSize }) => `Max ${formatSize(maxSize ?? 0)}`,
};
const props = {
  accept: "image/*,.pdf",
  maxFiles: 3,
  maxSize: 1024,
  messages,
  multiple: true,
  sizeStandard: "iec",
  validate: validator,
} satisfies FileUploadRootProps;

formatFileSize(10, { standard: "si" });
validateFiles({ candidates: [], current: [], validate: objectValidator });
validateFiles({ candidates: [], current: [], validate: messageValidator });
matchAcceptType("image/png", parseAccept("image/*"));

// @ts-expect-error custom messages cannot be registered for the free-form `custom` code
const invalidMessages: FileUploadMessages = { custom: () => "x" };
// @ts-expect-error size standards are closed
formatFileSize(10, { standard: "binary" });
// @ts-expect-error the model holds File objects, not names
const invalidModel: FileUploadRootProps = { modelValue: ["a.txt"] };
// @ts-expect-error paste scopes are closed
const invalidScope: FileUploadPasteScope = "window";
// @ts-expect-error validators receive File objects
const invalidValidator: FileUploadValidator = (file: string) => file;
// @ts-expect-error rejection codes are closed
const invalidCode: FileUploadRejectionCode = "file-missing";
// @ts-expect-error addFiles requires files
root.addFiles(["a.txt"]);

void [
  FileUploadClear,
  FileUploadDropzone,
  FileUploadItem,
  FileUploadItemDelete,
  FileUploadItemGroup,
  FileUploadItemName,
  FileUploadItemPreview,
  FileUploadItemSize,
  FileUploadTrigger,
  props,
  invalidMessages,
  invalidModel,
  invalidScope,
  invalidValidator,
  invalidCode,
];

// @ts-expect-error reject drag effects are a closed union
const invalidEffect: FileUploadRejectDragEffect = "move";
void invalidEffect;
