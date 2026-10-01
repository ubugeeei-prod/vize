<template>
  <main>
    <template v-if="show">
      <strong>{{ heading }}</strong>
    </template>
    <template v-else>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <p v-html="html" />
    </template>
    <!-- oxlint-disable vue/require-v-for-key -->
    <ul>
      <li v-for="item in items">
        <!-- oxlint-enable vue/require-v-for-key -->
        {{ item }}
      </li>
    </ul>
  </main>
</template>
