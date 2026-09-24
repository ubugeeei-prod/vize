export const fileUploadRendererFixtures = [
  {
    filename: "FileUploadConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
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
} from "./families/form/file-upload/file-upload.ts";
import type { FileUploadRejection } from "./families/form/file-upload/file-upload.ts";

const files = ref<readonly File[]>([]);
const rejected = ref(0);

function onReject(rejections: readonly FileUploadRejection[]): void {
  rejected.value += rejections.length;
}
</script>

<template>
  <FileUploadRoot
    v-model="files"
    accept="image/*,.pdf"
    multiple
    :max-files="4"
    :max-size="5000000"
    name="attachments"
    @reject="onReject"
  >
    <FileUploadDropzone v-slot="{ state }" aria-describedby="upload-hint" paste="self">
      <span>Drop files ({{ state }})</span>
      <FileUploadTrigger>Browse</FileUploadTrigger>
    </FileUploadDropzone>
    <p id="upload-hint">Images or PDF up to 5 MB. Rejected: {{ rejected }}</p>
    <FileUploadItemGroup v-slot="{ items }" aria-label="Attachments">
      <FileUploadItem v-for="item in items" :key="item.key" :file="item.file">
        <FileUploadItemPreview />
        <FileUploadItemName />
        <FileUploadItemSize />
        <FileUploadItemDelete :aria-label="'Remove ' + item.name">Remove</FileUploadItemDelete>
      </FileUploadItem>
    </FileUploadItemGroup>
    <FileUploadClear>Clear all</FileUploadClear>
  </FileUploadRoot>
</template>
`,
  },
] as const;
