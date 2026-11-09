<template>
  <main>
    <RouterLink :to="{ name: 'search', params: { query: 'vue' } }">search</RouterLink>
    <RouterLink :to="{ name: 'user-post', params: { userId: 1, postId: 42 } }">a post</RouterLink>
  </main>
</template>
