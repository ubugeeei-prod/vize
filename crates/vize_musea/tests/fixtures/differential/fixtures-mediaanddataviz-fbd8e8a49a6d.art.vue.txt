<script setup lang="ts">
import { computed } from "vue";
import { Swiper, SwiperSlide } from "swiper/vue";
import "swiper/css";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import {
  CategoryScale,
  Chart as ChartJS,
  Legend,
  LinearScale,
  LineElement,
  PointElement,
  Tooltip,
  type ChartData,
  type ChartOptions,
} from "chart.js";
import { Line } from "vue-chartjs";
import { RecycleScroller } from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import type { ChartPoint } from "./types";

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend);

const slides = [
  { id: "swiper", title: "Swiper.js", body: "Carousel coverage for Vue components." },
  { id: "tiptap", title: "Tiptap Vue", body: "Editor content and commands." },
  { id: "chart", title: "Vue Chart.js", body: "Chart data and options." },
];

const points = [
  { month: "Jan", value: 12 },
  { month: "Feb", value: 21 },
  { month: "Mar", value: 18 },
  { month: "Apr", value: 32 },
] satisfies ChartPoint[];

const virtualItems = Array.from({ length: 32 }, (_, index) => ({
  id: `product-${index}`,
  name: `Virtual product ${index + 1}`,
  downloads: (index + 1) * 125,
}));

const editor = useEditor({
  extensions: [StarterKit],
  content: "<p><strong>Vize</strong> checks Tiptap Vue editor content.</p>",
});

const editorPreview = computed(() => editor.value?.getText() ?? "");

const chartData = computed<ChartData<"line">>(() => ({
  labels: points.map((point) => point.month),
  datasets: [
    {
      label: "Ecosystem usage",
      data: points.map((point) => point.value),
      borderColor: "#0f766e",
      backgroundColor: "rgba(15, 118, 110, 0.18)",
      tension: 0.3,
    },
  ],
}));

const chartOptions: ChartOptions<"line"> = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: false,
    },
  },
  scales: {
    y: {
      beginAtZero: true,
    },
  },
};
</script>

<template>
  <section class="media-grid" aria-label="Media and data visualization coverage">
    <Swiper class="slider" :slides-per-view="1" :space-between="12">
      <SwiperSlide v-for="slide in slides" :key="slide.id">
        <article class="slide">
          <h2>{{ slide.title }}</h2>
          <p>{{ slide.body }}</p>
        </article>
      </SwiperSlide>
    </Swiper>

    <article class="editor-panel">
      <div class="toolbar">
        <button type="button" @click="editor?.chain().focus().toggleBold().run()">Bold</button>
        <button type="button" @click="editor?.chain().focus().toggleItalic().run()">Italic</button>
      </div>
      <EditorContent :editor="editor!" />
      <p>Preview: {{ editorPreview }}</p>
    </article>

    <article class="chart-panel">
      <Line :data="chartData" :options="chartOptions" />
    </article>

    <RecycleScroller
      class="virtual-list"
      :items="virtualItems"
      :item-size="44"
      key-field="id"
      v-slot="{ item }"
    >
      <div class="virtual-row">
        <strong>{{ item.name }}</strong>
        <span>{{ item.downloads }}</span>
      </div>
    </RecycleScroller>
  </section>
</template>

<style scoped>
.media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 12px;
}

.slide,
.editor-panel,
.chart-panel,
.virtual-list {
  border: 1px solid #d0d7de;
  border-radius: 8px;
  min-height: 180px;
  padding: 12px;
}

.chart-panel {
  height: 240px;
}

.virtual-list {
  height: 240px;
}

.virtual-row,
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
