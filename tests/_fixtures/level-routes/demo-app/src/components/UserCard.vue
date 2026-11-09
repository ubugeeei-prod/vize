<script setup lang="ts">
import { useRouter } from "vue-router";

defineProps<{ user: { id: number; name: string; tags: string[] } }>();

const router = useRouter();

function openLatestPost(userId: number, postId: number) {
  router.push({ name: "user-posts", params: { userId, postId } });
}
</script>

<template>
  <article class="user-card">
    <RouterLink :to="{ name: 'user', params: { id: user.id } }">{{ user.name }}</RouterLink>
    <RouterLink :to="{ name: 'user-tags', params: { userId: user.id, tags: user.tags } }">
      tags
    </RouterLink>
    <button @click="$router.push({ name: 'search', params: { query: ['vue', 'fes'] } })">
      search
    </button>
    <button @click="openLatestPost(user.id, 1)">latest post</button>
  </article>
</template>
