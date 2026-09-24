<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, toRaw, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { fileUploadContext } from "./file-upload-context.ts";
import type { FileUploadContextValue } from "./file-upload-context.ts";
import { collectTransferFiles, isTransferRejected } from "./file-upload-transfer.ts";
import type { FileUploadTransfer } from "./file-upload-transfer.ts";
import type {
  FileUploadAddResult,
  FileUploadItemSlotState,
  FileUploadRootEmits,
  FileUploadRootExpose,
  FileUploadRootProps,
  FileUploadSlotState,
  FileUploadSource,
  FileUploadState,
} from "./file-upload-types.ts";
import {
  effectiveMaxFiles,
  formatFileSize,
  matchesAccept,
  parseAccept,
  validateFiles,
} from "./file-upload-validation.ts";

const EMPTY: readonly File[] = Object.freeze([]);

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  accept = undefined,
  multiple = false,
  maxFiles = undefined,
  maxSize = undefined,
  minSize = undefined,
  disabled = false,
  required = false,
  name = undefined,
  form = undefined,
  directory = false,
  capture = undefined,
  validate = undefined,
  messages = undefined,
  locale = undefined,
  sizeStandard = "si",
  formatSize = undefined,
  previewAccept = "image/*",
} = defineProps<FileUploadRootProps>();

const emit = defineEmits<FileUploadRootEmits>();

defineSlots<{
  /** Compound FileUpload parts. Receives the file list, availability, and actions. */
  default(props: FileUploadSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const inputElement = useTemplateRef<HTMLInputElement>("inputElement");
const baseId = useDeterministicId({ id: () => id, hint: "file-upload" });
const inputId = computed(() => deriveDeterministicId(baseId.value, "input"));
const resolvedLocale = computed(() => locale ?? "en-US");
const valueState = useControllableState<readonly File[]>({
  value: () => modelValue,
  defaultValue: () => defaultValue ?? EMPTY,
  equals: sameFiles,
});
const files = computed(() => rawFiles(valueState.value.value));
const fileCount = computed<number>(() => files.value.length);
const disabledState = computed(() => disabled);
const dragging = shallowRef(false);
const maxFileCount = computed(() => effectiveMaxFiles(multiple, maxFiles));
const canAdd = computed(
  () => !disabledState.value && (!multiple || files.value.length < maxFileCount.value),
);
const state = computed<FileUploadState>(() => {
  if (disabledState.value) return "disabled";
  return files.value.length === 0 ? "empty" : "filled";
});
const previewTokens = computed(() => parseAccept(previewAccept));
const inputAttributes = computed<Record<string, string | undefined>>(() => ({
  capture,
  webkitdirectory: directory ? "" : undefined,
}));

const relativePaths = new Map<File, string>();
const previewUrls = new Map<File, string>();
const mounted = shallowRef(false);
const focusFallbacks: (() => HTMLElement | null)[] = [];
const fileKeys = new WeakMap<File, string>();
let nextFileKey = 0;

function getFileKey(file: File): string {
  let key = fileKeys.get(file);
  if (key === undefined) {
    nextFileKey += 1;
    key = `file-${nextFileKey}`;
    fileKeys.set(file, key);
  }
  return key;
}

function isPreviewable(file: File): boolean {
  return matchesAccept(file, previewTokens.value);
}

/** Strip Vue proxies so identity checks, Map keys, and object URLs see the consumer's File. */
function rawFiles(list: readonly File[]): readonly File[] {
  const raw = toRaw(list);
  return raw.some((file) => toRaw(file) !== file)
    ? Object.freeze(raw.map((file) => toRaw(file)))
    : raw;
}

function sameFiles(left: readonly File[], right: readonly File[]): boolean {
  if (left.length !== right.length) return false;
  return left.every((file, index) => {
    const other = right[index];
    return other !== undefined && toRaw(file) === toRaw(other);
  });
}

function formatBytes(bytes: number): string {
  if (formatSize) return formatSize(bytes, resolvedLocale.value);
  return formatFileSize(bytes, { locale: resolvedLocale.value, standard: sizeStandard });
}

function currentFiles(): readonly File[] {
  return files.value;
}

function currentInput(): HTMLInputElement | null {
  return inputElement.value;
}

function setFiles(next: readonly File[]): boolean {
  const previous = currentFiles();
  if (sameFiles(previous, next)) return false;
  const frozen = Object.freeze(next.map((file) => toRaw(file)));
  valueState.set(frozen);
  emit("update:modelValue", frozen);
  emit("change", frozen, previous);
  return true;
}

function addFiles(
  candidates: Iterable<File>,
  source: FileUploadSource,
  paths?: ReadonlyMap<File, string>,
): FileUploadAddResult {
  if (disabledState.value) return { accepted: EMPTY, rejected: [] };
  const result = validateFiles({
    accept,
    candidates: Array.from(candidates, (file) => toRaw(file)),
    current: files.value,
    formatSize: formatBytes,
    maxFiles,
    maxSize,
    messages,
    minSize,
    multiple,
    validate,
  });
  for (const file of result.accepted) {
    const path = paths?.get(file);
    if (path !== undefined) relativePaths.set(file, path);
  }
  setFiles(result.files);
  if (result.accepted.length > 0) emit("accept", result.accepted, source);
  if (result.rejected.length > 0) emit("reject", result.rejected, source);
  return { accepted: result.accepted, rejected: result.rejected };
}

async function addTransfer(
  transfer: FileUploadTransfer | null | undefined,
  source: FileUploadSource,
): Promise<FileUploadAddResult> {
  if (disabledState.value) return { accepted: EMPTY, rejected: [] };
  const collected = await collectTransferFiles(transfer);
  if (collected.files.length === 0) return { accepted: EMPTY, rejected: [] };
  return addFiles(collected.files, source, collected.paths);
}

function removeFile(file: File): boolean {
  if (disabledState.value) return false;
  const target = toRaw(file);
  return setFiles(files.value.filter((candidate) => candidate !== target));
}

function clear(): boolean {
  if (disabledState.value) return false;
  return setFiles(EMPTY);
}

function reset(): boolean {
  return setFiles(defaultValue ?? EMPTY);
}

function openPicker(): void {
  if (disabledState.value) return;
  inputElement.value?.click();
}

function getRelativePath(file: File): string | null {
  return relativePaths.get(toRaw(file)) ?? null;
}

function getPreviewUrl(file: File): string | null {
  if (!mounted.value || typeof URL.createObjectURL !== "function") return null;
  if (!isPreviewable(file)) return null;
  const existing = previewUrls.get(file);
  if (existing !== undefined) return existing;
  const url = URL.createObjectURL(file);
  previewUrls.set(file, url);
  return url;
}

function revokePreview(file: File): void {
  const url = previewUrls.get(file);
  if (url === undefined) return;
  previewUrls.delete(file);
  if (typeof URL.revokeObjectURL === "function") URL.revokeObjectURL(url);
}

function getItemState(item: File): FileUploadItemSlotState {
  const file = toRaw(item);
  return {
    disabled: disabledState.value,
    file,
    formattedSize: formatBytes(file.size),
    index: files.value.indexOf(file),
    key: getFileKey(file),
    name: file.name,
    previewUrl: getPreviewUrl(file),
    relativePath: getRelativePath(file),
    size: file.size,
    type: file.type,
  };
}

function syncInput(): void {
  const input = currentInput();
  if (!input || typeof DataTransfer !== "function") return;
  try {
    const transfer = new DataTransfer();
    for (const file of files.value) transfer.items.add(file);
    input.files = transfer.files;
  } catch {
    // Some engines expose DataTransfer without a writable FileList; submission then relies on
    // consumer-side uploading, which the reactive value already supports.
  }
}

function onInputChange(event: Event): void {
  const input = event.currentTarget;
  if (!(input instanceof HTMLInputElement)) return;
  const picked = input.files ? Array.from(input.files) : [];
  const paths = new Map<File, string>();
  for (const file of picked) {
    const path = typeof file.webkitRelativePath === "string" ? file.webkitRelativePath : "";
    if (path.length > 0) paths.set(file, path);
  }
  if (picked.length > 0) addFiles(picked, "input", paths);
  syncInput();
}

function onInvalid(event: Event): void {
  emit("invalid", event);
  focusFallback();
}

function setFocusFallback(target: () => HTMLElement | null): () => void {
  focusFallbacks.push(target);
  return () => {
    const index = focusFallbacks.indexOf(target);
    if (index >= 0) focusFallbacks.splice(index, 1);
  };
}

function focusFallback(): void {
  for (const target of focusFallbacks) {
    const candidate = target();
    if (candidate && candidate.isConnected) {
      candidate.focus();
      return;
    }
  }
}

watch(
  files,
  (next, previous) => {
    for (const file of previous ?? EMPTY) {
      if (next.includes(file)) continue;
      revokePreview(file);
      relativePaths.delete(file);
    }
    syncInput();
  },
  { flush: "post" },
);

onMounted(() => {
  mounted.value = true;
  syncInput();
});

onScopeDispose(() => {
  for (const file of previewUrls.keys()) revokePreview(file);
  relativePaths.clear();
});

const slotState = computed<FileUploadSlotState>(() => ({
  canAdd: canAdd.value,
  clear,
  disabled: disabledState.value,
  dragging: dragging.value,
  files: files.value,
  openPicker,
  removeFile,
  state: state.value,
}));

fileUploadContext.provide({
  addFiles,
  addTransfer,
  canAdd,
  clear,
  disabled: disabledState,
  dragging,
  files,
  focusFallback,
  getItemState,
  id: baseId,
  isPreviewable,
  isTransferRejected: (transfer) =>
    isTransferRejected(transfer, {
      accept,
      currentCount: files.value.length,
      maxFiles,
      multiple,
    }),
  openPicker,
  removeFile,
  setFocusFallback,
  state,
} satisfies FileUploadContextValue);

type FileUploadRootSetupExpose = Omit<
  FileUploadRootExpose,
  "element" | "files" | "id" | "inputElement" | "state"
> & {
  readonly element: typeof element;
  readonly files: ComputedRef<readonly File[]>;
  readonly id: ComputedRef<string>;
  readonly inputElement: typeof inputElement;
  readonly state: ComputedRef<FileUploadState>;
};

const exposed = {
  addFiles: (candidates: Iterable<File>) => addFiles(candidates, "api"),
  clear,
  element,
  files,
  getRelativePath,
  id: baseId,
  inputElement,
  openPicker,
  removeFile,
  reset,
  state,
} satisfies FileUploadRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    data-vize-ui="file-upload-root"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-dragging="dragging ? 'true' : undefined"
    :data-multiple="multiple ? 'true' : undefined"
    :data-count="fileCount"
  >
    <slot v-bind="slotState" />
    <input
      :id="inputId"
      ref="inputElement"
      type="file"
      hidden
      tabindex="-1"
      :name
      :form
      :accept
      :multiple="multiple || directory"
      :required="required && fileCount === 0"
      :disabled="disabledState"
      v-bind="inputAttributes"
      data-vize-ui="file-upload-input"
      part="input"
      @change="onInputChange"
      @invalid="onInvalid"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
