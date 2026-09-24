<script setup lang="ts">
import "../theme.css";
defineProps<{
  title: string;
  description?: string;
  image?: string;
  variant?: "default" | "outlined" | "elevated";
}>();

defineArt("./MuseaCard.vue", {
  title: "Card",
  category: "Layout",
  tags: ["card", "container", "layout"],
  status: "ready",
});
</script>

<template>
  <div class="card" :class="`card--${variant ?? 'default'}`">
    <div v-if="image" class="card-image">
      <div class="card-image-placeholder" :style="{ background: image }">
        <span>{{ title.charAt(0) }}</span>
      </div>
    </div>
    <div class="card-body">
      <h3 class="card-title">{{ title }}</h3>
      <p v-if="description" class="card-description">{{ description }}</p>
      <div class="card-footer">
        <slot />
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  border-radius: 8px;
  overflow: hidden;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease;
  max-width: 320px;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
}

.card:hover {
  transform: translateY(-2px);
}

.card--default {
  background: var(--musea-surface);
  border: 1px solid var(--musea-line);
}

.card--outlined {
  background: transparent;
  border: 2px solid var(--musea-line);
}

.card--elevated {
  background: var(--musea-surface);
  border: none;
  box-shadow: var(--musea-shadow-card);
}

.card--elevated:hover {
  box-shadow: var(--musea-shadow-card-hover);
}

.card-image {
  aspect-ratio: 16 / 9;
  overflow: hidden;
}

.card-image-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  font-weight: 700;
  color: var(--musea-card-image-text);
}

.card-body {
  padding: 1rem 1.25rem;
}

.card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--musea-ink);
  margin: 0 0 0.5rem;
}

.card-description {
  font-size: 0.875rem;
  color: var(--musea-body-muted);
  margin: 0 0 1rem;
  line-height: 1.5;
}

.card-footer {
  display: flex;
  gap: 0.5rem;
}
</style>

<art>
  <variant name="Default" default>
    <Self title="Getting Started" description="Learn how to build with Musea components.">
      <button class="btn btn--primary">Read More</button>
    </Self>
  </variant>
  <variant name="With Image">
    <Self
      title="Featured"
      description="A card with an image header for rich content display."
      image="var(--musea-accent)"
    >
      <button class="btn btn--primary">View</button>
      <button class="btn">Share</button>
    </Self>
  </variant>
  <variant name="Outlined">
    <Self title="Outlined Card" description="A card with an outlined style." variant="outlined">
      <button class="btn">Action</button>
    </Self>
  </variant>
  <variant name="Elevated">
    <Self title="Elevated Card" description="A card with elevation shadow." variant="elevated">
      <button class="btn btn--primary">Action</button>
    </Self>
  </variant>
</art>
