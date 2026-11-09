<script setup lang="coffee">
import { ref } from "vue"

defineProps ["title"]

count = ref 0
done = ref no
name = ref "world"
items = ref ["write", "compile", "ship"]

# Clicking past five marks the counter done.
increment = ->
  count.value += 1
  done.value = count.value > 5
</script>

<template>
  <section class="counter">
    <h1>{{ title }}</h1>
    <button type="button" :class="if done then 'hot' else 'cold'" @click="increment">
      Clicked {{ count }} times
    </button>
    <p v-if="count > 3 and not done">Hot!</p>
    <p v-else>Keep clicking</p>
    <input v-model="name" />
    <p>{{ "Hello, #{name}!" }}</p>
    <ul>
      <li v-for="item in items">{{ item.toUpperCase() }}</li>
    </ul>
  </section>
</template>
