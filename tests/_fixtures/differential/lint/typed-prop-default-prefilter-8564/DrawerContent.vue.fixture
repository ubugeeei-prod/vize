<script setup lang="ts">
import { DrawerContent, type DrawerDirection } from 'vaul-vue'

const props = withDefaults(defineProps<{
  direction: DrawerDirection
}>(), {
  direction: 'bottom',
})
</script>

<template>
  <DrawerContent
    data-testid="content"
    class="bg-zinc-100 flex fixed p-6"
    :class="{
      'rounded-t-[10px] flex-col h-[50%] bottom-0 left-0 right-0': direction === 'bottom',
      'rounded-b-[10px] flex-col h-[50%] top-0 left-0 right-0': direction === 'top',
      'rounded-r-[10px] flex-row w-[50%] left-0 top-0 bottom-0': direction === 'left',
      'rounded-l-[10px] flex-row w-[50%] right-0 top-0 bottom-0': direction === 'right',
    }"
  >
    <div
      class="w-full h-full flex rounded-full gap-8"
      :class="{
        'flex-col': direction === 'bottom',
        'flex-col-reverse': direction === 'top',
        'flex-row-reverse': direction === 'left',
        'flex-row ': direction === 'right',
      }"
    >
      <div
        class="rounded-full bg-zinc-300"
        :class="{
          'mx-auto w-12 h-1.5': direction === 'top' || direction === 'bottom',
          'my-auto h-12 w-1.5': direction === 'left' || direction === 'right',
        }"
      />

      <div class="grid place-content-center w-full h-full">
        <slot />
      </div>
    </div>
  </DrawerContent>
</template>
