<script setup lang="ts">
defineProps<{
  direction: "horizontal" | "vertical";
  isResizing?: boolean;
}>();

const emit = defineEmits<{
  (e: "pointerdown", event: PointerEvent): void;
}>();
</script>

<template>
  <div
    :class="[
      'resize-handle',
      `resize-handle--${direction}`,
      { 'resize-handle--active': isResizing },
    ]"
    @pointerdown.stop.prevent="(event) => emit('pointerdown', event)"
  >
    <div class="resize-handle__indicator" />
  </div>
</template>

<style scoped>
.resize-handle {
  position: relative;
  flex-shrink: 0;
  background: transparent;
  transition: background-color 0.15s;
  z-index: var(--musea-z-resize, 10);
  touch-action: none;
}

.resize-handle--horizontal {
  width: 10px;
  cursor: col-resize;
}

.resize-handle--vertical {
  height: 10px;
  cursor: row-resize;
}

.resize-handle:hover,
.resize-handle--active {
  background: color-mix(in srgb, var(--musea-accent) 30%, transparent);
}

.resize-handle__indicator {
  position: absolute;
  background: var(--musea-border);
  transition: background-color 0.15s;
}

.resize-handle--horizontal {
  .resize-handle__indicator {
    inset-inline-start: 50%;
    inset-block-start: 0;
    inset-block-end: 0;
    width: 1px;
    transform: translateX(-50%);
  }
}

.resize-handle--vertical {
  .resize-handle__indicator {
    inset-block-start: 50%;
    inset-inline-start: 0;
    inset-inline-end: 0;
    height: 1px;
    transform: translateY(-50%);
  }
}

.resize-handle:hover,
.resize-handle--active {
  .resize-handle__indicator {
    background: var(--musea-accent);
  }
}
</style>
