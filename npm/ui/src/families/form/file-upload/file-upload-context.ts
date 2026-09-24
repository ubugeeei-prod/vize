import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { FileUploadTransfer } from "./file-upload-transfer.ts";
import type {
  FileUploadAddResult,
  FileUploadItemSlotState,
  FileUploadSource,
  FileUploadState,
} from "./file-upload-types.ts";

/** Shared state and actions for the FileUpload compound parts. */
export interface FileUploadContextValue {
  readonly id: ComputedRef<string>;
  readonly files: ComputedRef<readonly File[]>;
  readonly disabled: ComputedRef<boolean>;
  readonly canAdd: ComputedRef<boolean>;
  readonly dragging: ShallowRef<boolean>;
  readonly state: ComputedRef<FileUploadState>;
  readonly openPicker: () => void;
  readonly addFiles: (
    files: Iterable<File>,
    source: FileUploadSource,
    paths?: ReadonlyMap<File, string>,
  ) => FileUploadAddResult;
  readonly addTransfer: (
    transfer: FileUploadTransfer | null | undefined,
    source: FileUploadSource,
  ) => Promise<FileUploadAddResult>;
  readonly isTransferRejected: (transfer: FileUploadTransfer | null | undefined) => boolean;
  readonly removeFile: (file: File) => boolean;
  readonly clear: () => boolean;
  readonly getItemState: (file: File) => FileUploadItemSlotState;
  readonly isPreviewable: (file: File) => boolean;
  readonly setFocusFallback: (element: () => HTMLElement | null) => () => void;
  readonly focusFallback: () => void;
}

export const fileUploadContext = createContext<FileUploadContextValue>("FileUpload");

/** Per-item state shared from FileUploadItem to its parts. */
export interface FileUploadItemContextValue {
  readonly state: ComputedRef<FileUploadItemSlotState>;
}

export const fileUploadItemContext = createContext<FileUploadItemContextValue>("FileUploadItem");
