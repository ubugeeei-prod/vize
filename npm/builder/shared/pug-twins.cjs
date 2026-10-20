// One component written twice: with `<template lang="pug">` and with the
// equivalent HTML template. Bundler tests render both and compare the HTML.
"use strict";

const script = `<script setup>
import { ref } from "vue";
const title = "Pug";
const items = ["a", "b"];
const ok = ref(true);
</script>
`;

const pug = `${script}
<template lang="pug">
section#app.card(:class="{ active: ok }")
  h1.title {{ title }}
  ul
    li(v-for="item in items" :key="item") {{ item }}
  p(v-if="ok") ready
  p(v-else) waiting
  a(href="/docs" target="_blank") Docs
</template>
`;

const jade = pug.replace('lang="pug"', 'lang="jade"');

const html = `${script}
<template>
  <section id="app" class="card" :class="{ active: ok }">
    <h1 class="title">{{ title }}</h1>
    <ul>
      <li v-for="item in items" :key="item">{{ item }}</li>
    </ul>
    <p v-if="ok">ready</p>
    <p v-else>waiting</p>
    <a href="/docs" target="_blank">Docs</a>
  </section>
</template>
`;

module.exports = { pug, jade, html };
