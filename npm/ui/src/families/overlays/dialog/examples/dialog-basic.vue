<!-- Modal dialog opened from a trigger, labelled by its title, and closed from inside. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
} from "../dialog.ts";

const open = ref(false);
</script>

<template>
  <Dialog v-model:open="open">
    <DialogTrigger>Edit profile</DialogTrigger>
    <DialogPortal>
      <DialogOverlay />
      <DialogContent>
        <DialogTitle>Edit profile</DialogTitle>
        <DialogDescription>Update how your name appears to teammates.</DialogDescription>
        <label>
          Display name
          <input name="displayName" autocomplete="nickname" />
        </label>
        <DialogClose>Save changes</DialogClose>
      </DialogContent>
    </DialogPortal>
  </Dialog>
</template>
