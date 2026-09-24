<!-- Side drawer with a drag handle that holds secondary filter controls. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerHandle,
  DrawerTitle,
  DrawerTrigger,
} from "../drawer.ts";

const open = ref(false);
</script>

<template>
  <Drawer v-model:open="open" side="right">
    <DrawerTrigger>Open filters</DrawerTrigger>
    <DrawerContent>
      <DrawerHandle />
      <DrawerTitle>Filters</DrawerTitle>
      <DrawerDescription>Narrow the order list by status.</DrawerDescription>
      <fieldset>
        <legend>Status</legend>
        <label><input type="checkbox" name="status" value="paid" /> Paid</label>
        <label><input type="checkbox" name="status" value="refunded" /> Refunded</label>
      </fieldset>
      <DrawerClose>Apply filters</DrawerClose>
    </DrawerContent>
  </Drawer>
</template>
