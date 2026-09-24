/** Machine-readable reason a candidate file was not added to a FileUpload. */
export type FileUploadRejectionCode =
  | "custom"
  | "file-invalid-type"
  | "file-too-large"
  | "file-too-small"
  | "too-many-files";

/** Built-in rejection reasons that ship a default message. */
export type FileUploadBuiltInRejectionCode = Exclude<FileUploadRejectionCode, "custom">;

/** How candidate files reached a FileUpload. */
export type FileUploadSource = "api" | "drop" | "input" | "paste";

/** Unit convention used by the default size formatter. */
export type FileUploadSizeStandard = "iec" | "si";

/** Where a dropzone listens for clipboard files. */
export type FileUploadPasteScope = "document" | "none" | "self";

/** `dropEffect` announced while a drag preview is rejected. `none` refuses the drop. */
export type FileUploadRejectDragEffect = "copy" | "none";

/** State token exposed by the FileUpload root and list parts. */
export type FileUploadState = "disabled" | "empty" | "filled";

/** State token exposed by FileUploadDropzone. */
export type FileUploadDropzoneState = "disabled" | "dragging" | "idle" | "rejecting";

/** State token exposed by FileUploadItemPreview. */
export type FileUploadPreviewState = "pending" | "ready" | "unsupported";

/** One validation failure for one candidate file. */
export interface FileUploadError {
  /** Machine-readable failure reason. */
  readonly code: FileUploadRejectionCode;

  /** Human-readable failure message. */
  readonly message: string;
}

/** A candidate file together with every reason it was rejected. */
export interface FileUploadRejection {
  /** The rejected candidate file. */
  readonly file: File;

  /** Every validation failure, in check order. */
  readonly errors: readonly FileUploadError[];
}

/**
 * Result of a custom validator. `null`/`undefined` accept the file; a built-in code reuses its
 * default message; any other string becomes a `custom` error message; objects pass through.
 */
export type FileUploadValidationResult =
  | FileUploadError
  | FileUploadRejectionCode
  | (string & {})
  | null
  | undefined;

/** Custom validation hook run after the built-in type and size checks. */
export type FileUploadValidator = (
  file: File,
  accepted: readonly File[],
) => FileUploadValidationResult;

/** Limits and file data passed to message factories. */
export interface FileUploadMessageDetail {
  /** Candidate file being described. */
  readonly file: File;

  /** Native `accept` string configured on the root. */
  readonly accept: string | undefined;

  /** Maximum byte size, when configured. */
  readonly maxSize: number | undefined;

  /** Minimum byte size, when configured. */
  readonly minSize: number | undefined;

  /** Maximum accepted file count. */
  readonly maxFiles: number;

  /** Formats a byte count with the root size formatter. */
  readonly formatSize: (bytes: number) => string;
}

/** Message factories that replace the default English rejection messages. */
export type FileUploadMessages = {
  readonly [Code in FileUploadBuiltInRejectionCode]?: (detail: FileUploadMessageDetail) => string;
};

/** Byte-size formatter used by FileUploadItemSize and rejection messages. */
export type FileUploadSizeFormatter = (bytes: number, locale: string) => string;

/** Outcome of one {@link FileUploadRootExpose.addFiles} request. */
export interface FileUploadAddResult {
  /** Files appended to (or, for single uploads, replacing) the value. */
  readonly accepted: readonly File[];

  /** Candidate files that failed validation. */
  readonly rejected: readonly FileUploadRejection[];
}

/** Props accepted by FileUploadRoot. */
export interface FileUploadRootProps {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled file list. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: readonly File[];

  /**
   * Initial file list for uncontrolled use.
   *
   * @default []
   */
  readonly defaultValue?: readonly File[];

  /**
   * Native `accept` list: MIME types, `type/*` wildcards, and `.ext` suffixes.
   *
   * @default undefined
   */
  readonly accept?: string;

  /**
   * Allow more than one file. Single uploads replace the current file.
   *
   * @default false
   */
  readonly multiple?: boolean;

  /**
   * Maximum number of files held at once. Ignored unless `multiple` is set.
   *
   * @default Infinity
   */
  readonly maxFiles?: number;

  /**
   * Maximum accepted byte size per file.
   *
   * @default undefined
   */
  readonly maxSize?: number;

  /**
   * Minimum accepted byte size per file.
   *
   * @default undefined
   */
  readonly minSize?: number;

  /**
   * Disable picking, dropping, pasting, and removing files.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Require at least one file for native form validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Native input name used for form submission.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the root's ancestor chain.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Let the native picker choose whole directories (`webkitdirectory`).
   *
   * @default false
   */
  readonly directory?: boolean;

  /**
   * Native capture hint for mobile camera and microphone pickers.
   *
   * @default undefined
   */
  readonly capture?: "environment" | "user";

  /**
   * Custom validation run after the built-in type and size checks.
   *
   * @default undefined
   */
  readonly validate?: FileUploadValidator;

  /**
   * Replacement factories for built-in rejection messages.
   *
   * @default undefined
   */
  readonly messages?: FileUploadMessages;

  /**
   * BCP 47 locale used for size formatting. `undefined` formats with `en-US`, keeping
   * server and client output identical; pass the app locale (e.g. from `useLocale()`).
   *
   * @default undefined
   */
  readonly locale?: string;

  /**
   * Unit convention used by the default size formatter.
   *
   * @default "si"
   */
  readonly sizeStandard?: FileUploadSizeStandard;

  /**
   * Replacement byte-size formatter.
   *
   * @default undefined
   */
  readonly formatSize?: FileUploadSizeFormatter;

  /**
   * `accept`-syntax filter for files that receive client-side object-URL previews.
   *
   * @default "image/*"
   */
  readonly previewAccept?: string;
}

/** Events emitted by FileUploadRoot. */
export interface FileUploadRootEmits {
  /** Requested file list for `v-model`. */
  "update:modelValue": [files: readonly File[]];

  /** Fired after any distinct file-list request. */
  change: [files: readonly File[], previous: readonly File[]];

  /** Fired with the files accepted from one pick, drop, paste, or API request. */
  accept: [files: readonly File[], source: FileUploadSource];

  /** Fired with the files rejected from one pick, drop, paste, or API request. */
  reject: [rejections: readonly FileUploadRejection[], source: FileUploadSource];

  /** Fired when native form validation reports the hidden input invalid. */
  invalid: [nativeEvent: Event];
}

/** State exposed to the FileUploadRoot default slot. */
export interface FileUploadSlotState {
  /** Current file list. */
  readonly files: readonly File[];

  /** Whether interaction is disabled. */
  readonly disabled: boolean;

  /** Whether a file drag is currently over a dropzone. */
  readonly dragging: boolean;

  /** Whether more files can be added. */
  readonly canAdd: boolean;

  /** Stable state token for styling and tests. */
  readonly state: FileUploadState;

  /** Opens the native file picker. */
  readonly openPicker: () => void;

  /** Removes one file. */
  readonly removeFile: (file: File) => boolean;

  /** Removes every file. */
  readonly clear: () => boolean;
}

/** Public instance exposed by FileUploadRoot. */
export interface FileUploadRootExpose {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Hidden native file input used for picking and form submission. */
  readonly inputElement: HTMLInputElement | null;

  /** Root-owned base id. */
  readonly id: string;

  /** Current file list. */
  readonly files: readonly File[];

  /** Stable state token. */
  readonly state: FileUploadState;

  /** Opens the native file picker unless disabled. */
  readonly openPicker: () => void;

  /** Validates and adds candidate files as a programmatic (`api`) request. */
  readonly addFiles: (files: Iterable<File>) => FileUploadAddResult;

  /** Removes one file and reports whether the list changed. */
  readonly removeFile: (file: File) => boolean;

  /** Removes every file and reports whether the list changed. */
  readonly clear: () => boolean;

  /** Restores the default file list and reports whether the list changed. */
  readonly reset: () => boolean;

  /** Relative path recorded for a file dropped inside a directory, or `null`. */
  readonly getRelativePath: (file: File) => string | null;
}

/** State exposed to FileUploadDropzone slots. */
export interface FileUploadDropzoneSlotState {
  /** Whether a file drag is over the dropzone. */
  readonly dragging: boolean;

  /** Whether the dragged items would be rejected. */
  readonly rejecting: boolean;

  /** Whether interaction is disabled. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: FileUploadDropzoneState;
}

/** Public instance exposed by FileUploadDropzone. */
export interface FileUploadDropzoneExpose extends FileUploadDropzoneSlotState {
  /** Rendered dropzone element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic dropzone id. */
  readonly id: string;

  /** Moves focus to the dropzone. */
  readonly focus: (options?: FocusOptions) => void;
}

/** State exposed to FileUploadTrigger and FileUploadClear slots. */
export interface FileUploadActionSlotState {
  /** Whether the action is unavailable. */
  readonly disabled: boolean;

  /** Current number of files. */
  readonly count: number;
}

/** Public instance exposed by FileUploadTrigger and FileUploadClear. */
export interface FileUploadActionExpose extends FileUploadActionSlotState {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Moves focus to the button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** State exposed to FileUploadItemGroup slots. */
export interface FileUploadItemGroupSlotState {
  /** Current file list, in value order. */
  readonly files: readonly File[];

  /** Per-file state in value order, with stable `key`s for `v-for`. */
  readonly items: readonly FileUploadItemSlotState[];

  /** Stable state token for styling and tests. */
  readonly state: FileUploadState;
}

/** Per-file state exposed to FileUploadItem and its parts. */
export interface FileUploadItemSlotState {
  /** Described file. */
  readonly file: File;

  /** Stable per-file key suitable for `v-for` keys. Never rendered to the DOM. */
  readonly key: string;

  /** Position of the file in the current value. */
  readonly index: number;

  /** File name. */
  readonly name: string;

  /** Byte size. */
  readonly size: number;

  /** MIME type, possibly empty. */
  readonly type: string;

  /** Size formatted with the root formatter. */
  readonly formattedSize: string;

  /** Relative path for files dropped inside a directory, or `null`. */
  readonly relativePath: string | null;

  /** Client-only object URL for previewable files; `null` during SSR and hydration. */
  readonly previewUrl: string | null;

  /** Whether interaction is disabled. */
  readonly disabled: boolean;
}

/** State exposed to FileUploadItemPreview slots. */
export interface FileUploadItemPreviewSlotState extends FileUploadItemSlotState {
  /** Stable preview state token. */
  readonly state: FileUploadPreviewState;
}
