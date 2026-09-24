<script setup lang="ts">
import "../theme.css";
import { computed } from "vue";

const props = defineProps<{
  name?: string;
  src?: string;
  size?: "sm" | "md" | "lg";
}>();

defineArt("./MuseaAvatar.vue", {
  title: "Avatar",
  category: "Components",
  tags: ["avatar", "user", "profile"],
  status: "ready",
});

const initials = computed(() => {
  if (!props.name) return "?";
  return props.name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
});

function safeUrl(value?: string): string | undefined {
  if (!value) return undefined;
  try {
    const url = new URL(value, "https://example.invalid");
    return url.protocol === "http:" || url.protocol === "https:" ? value : undefined;
  } catch {
    return undefined;
  }
}

const safeSrc = computed(() => safeUrl(props.src));
</script>

<template>
  <span class="avatar" :class="`avatar--${size ?? 'md'}`">
    <img v-if="safeSrc" :src="safeUrl(src)" :alt="name ?? 'avatar'" class="avatar-img" />
    <span v-else class="avatar-initials">{{ initials }}</span>
  </span>
</template>

<style scoped>
.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--musea-accent);
  color: var(--musea-paper);
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  font-weight: 600;
  overflow: hidden;
  flex-shrink: 0;
}

.avatar--sm {
  width: 28px;
  height: 28px;
  font-size: 0.625rem;
}

.avatar--md {
  width: 40px;
  height: 40px;
  font-size: 0.8125rem;
}

.avatar--lg {
  width: 56px;
  height: 56px;
  font-size: 1.125rem;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-initials {
  user-select: none;
}
</style>

<art>
  <variant name="Default" default>
    <Self name="Jane Doe" />
  </variant>
  <variant name="With Image">
    <Self name="Jane Doe" src="https://i.pravatar.cc/80?img=1" />
  </variant>
  <variant name="Sizes">
    <div style="display: flex; gap: 0.75rem; align-items: center">
      <Self name="SM" size="sm" />
      <Self name="MD" size="md" />
      <Self name="LG" size="lg" />
    </div>
  </variant>
  <variant name="Group">
    <div style="display: flex; margin-left: 0">
      <Self name="Alice" style="margin-left: 0; border: 2px solid var(--musea-paper)" />
      <Self name="Bob" style="margin-left: -8px; border: 2px solid var(--musea-paper)" />
      <Self name="Charlie" style="margin-left: -8px; border: 2px solid var(--musea-paper)" />
    </div>
  </variant>
</art>
