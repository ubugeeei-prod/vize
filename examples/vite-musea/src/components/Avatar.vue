<script setup lang="ts">
const props = defineProps<{
  name?: string;
  src?: string;
  size?: "sm" | "md" | "lg";
}>();

const initials = computed(() => {
  if (!props.name) return "?";
  return props.name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
});
</script>

<script lang="ts">
import { computed } from "vue";
</script>

<template>
  <span class="avatar" :class="`avatar--${size ?? 'md'}`">
    <img v-if="src" :src="src" :alt="name ?? 'avatar'" class="avatar-img" />
    <span v-else class="avatar-initials">{{ initials }}</span>
  </span>
</template>

<style scoped>
.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: #6b5090;
  color: #e6e2d6;
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

<art title="Avatar" category="Components" status="ready" tags="avatar,user,profile">
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
      <Self name="Alice" style="margin-left: 0; border: 2px solid #e6e2d6" />
      <Self name="Bob" style="margin-left: -8px; border: 2px solid #e6e2d6" />
      <Self name="Charlie" style="margin-left: -8px; border: 2px solid #e6e2d6" />
    </div>
  </variant>
</art>
