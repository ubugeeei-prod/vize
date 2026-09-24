<!-- Two draggable, resizable windows with title bars, window controls, and a taskbar dock. -->
<script setup lang="ts">
import { FloatingWindow, WindowDock, WindowManager } from "../window-manager.ts";

// The manager is the positioning area for its windows, so it needs a size.
const desktopSize = { blockSize: "24rem", inlineSize: "100%" };
</script>

<template>
  <WindowManager :snap-threshold="12" :style="desktopSize">
    <FloatingWindow
      id="notes"
      title="Notes"
      :default-rect="{ x: 16, y: 16, width: 280, height: 180 }"
    >
      <template #default="{ handleProps, minimize, toggleMaximize }">
        <div v-bind="handleProps">Notes</div>
        <button type="button" @click="minimize">Minimize notes</button>
        <button type="button" @click="toggleMaximize">Maximize notes</button>
        <p>Drag the title bar or use the arrow keys to move this window.</p>
      </template>
    </FloatingWindow>
    <FloatingWindow
      id="calendar"
      title="Calendar"
      :default-rect="{ x: 320, y: 48, width: 260, height: 200 }"
    >
      <template #default="{ handleProps, minimize }">
        <div v-bind="handleProps">Calendar</div>
        <button type="button" @click="minimize">Minimize calendar</button>
        <p>Shift with the arrow keys resizes the window.</p>
      </template>
    </FloatingWindow>
    <WindowDock label="Taskbar" />
  </WindowManager>
</template>
