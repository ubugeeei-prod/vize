<!-- Image upload with a dropzone, a browse button, and a removable file list with previews. -->
<script setup lang="ts">
import { shallowRef, useId } from "vue";

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
  FileUploadTrigger,
} from "../file-upload.ts";

const photos = shallowRef<readonly File[]>([]);
const hintId = useId();
</script>

<template>
  <FileUpload
    v-model="photos"
    name="photos"
    accept="image/*"
    multiple
    :max-files="5"
    :max-size="5_000_000"
  >
    <p :id="hintId">Up to five images, 5 MB each.</p>
    <FileUploadDropzone aria-label="Drop photos to upload" :aria-describedby="hintId">
      <FileUploadTrigger>Browse files</FileUploadTrigger>
    </FileUploadDropzone>
    <FileUploadItemGroup v-slot="{ items }" aria-label="Selected photos">
      <FileUploadItem v-for="item in items" :key="item.key" :file="item.file">
        <FileUploadItemPreview :alt="item.name" />
        <FileUploadItemName />
        <FileUploadItemSize />
        <FileUploadItemDelete :aria-label="`Remove ${item.name}`">Remove</FileUploadItemDelete>
      </FileUploadItem>
    </FileUploadItemGroup>
    <FileUploadClear>Remove all</FileUploadClear>
  </FileUpload>
</template>
