<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, useTemplateRef, watch } from "vue";
import type { Range } from "./offsets";

const { span, revision } = defineProps<{ span: Range | null; revision: string }>();
const root = useTemplateRef<SVGSVGElement>("root");
const links = ref<{ id: string; path: string; x1: number; y1: number; x2: number; y2: number }[]>(
  [],
);
let surface: HTMLElement | null = null;
let observer: ResizeObserver | null = null;
let frame = 0;

function draw() {
  frame = 0;
  if (!surface || !span) {
    links.value = [];
    return;
  }
  const bounds = surface.getBoundingClientRect();
  const panels = [".davinci-source", '[data-layer="l1"]', '[data-layer="l2"]', '[data-layer="l3"]'];
  const anchors = panels.map((selector, index) => {
    const panel = surface!.querySelector(selector);
    if (!panel) return null;
    const viewport = panel.querySelector(index === 0 ? ".davinci-editor" : ".davinci-folio");
    if (!viewport) return null;
    const clip = viewport.getBoundingClientRect();
    const rows = [
      ...panel.querySelectorAll(
        index === 0
          ? ".davinci-provenance"
          : index === 1
            ? ".dv-surface-unit.linked"
            : ".davinci-line.linked",
      ),
    ];
    const row = rows
      .map((node) => node.getBoundingClientRect())
      .find((rect) => rect.bottom > clip.top && rect.top < clip.bottom);
    if (!row) return null;
    const rect = panel.getBoundingClientRect();
    return {
      left: rect.left - bounds.left + 16,
      right: rect.right - bounds.left - 16,
      y: Math.max(clip.top + 4, Math.min(clip.bottom - 4, row.top + row.height / 2)) - bounds.top,
    };
  });
  links.value = anchors.flatMap((from, index) => {
    const to = anchors[index + 1];
    if (!from || !to) return [];
    const middle = (from.right + to.left) / 2;
    return [
      {
        id: `l${index + 1}`,
        x1: from.right,
        y1: from.y,
        x2: to.left,
        y2: to.y,
        path: `M ${from.right} ${from.y} C ${middle} ${from.y}, ${middle} ${to.y}, ${to.left} ${to.y}`,
      },
    ];
  });
}
function schedule() {
  if (frame) cancelAnimationFrame(frame);
  frame = requestAnimationFrame(draw);
}
watch(
  () => [span, revision],
  async () => {
    await nextTick();
    schedule();
  },
);
onMounted(() => {
  surface = root.value?.closest(".dv-workspace") ?? null;
  if (!surface) return;
  surface.addEventListener("scroll", schedule, true);
  observer = new ResizeObserver(schedule);
  observer.observe(surface);
  surface.querySelectorAll("[data-pane]").forEach((pane) => observer!.observe(pane));
  schedule();
});
onUnmounted(() => {
  surface?.removeEventListener("scroll", schedule, true);
  observer?.disconnect();
  cancelAnimationFrame(frame);
});
</script>

<template>
  <svg ref="root" class="dv-trace-links" aria-hidden="true">
    <g v-for="link in links" :key="link.id" :class="`dv-link-${link.id}`">
      <path class="dv-trace-outline" :d="link.path" />
      <path class="dv-trace-path" :d="link.path" />
      <circle :cx="link.x1" :cy="link.y1" r="4" />
      <circle :cx="link.x2" :cy="link.y2" r="4" />
    </g>
  </svg>
</template>
