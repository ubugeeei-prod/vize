<script setup lang="python">
from vue import ref

define_props(["title"])

count = ref(0)
done = ref(False)
name = ref("world")
items = ref(["write", "compile", "ship"])

# Clicking past five marks the counter done.
def increment():
    count.value += 1
    done.value = count.value > 5
</script>

<template>
  <section class="counter">
    <h1>{{ title }}</h1>
    <button type="button" :class="'hot' if done else 'cold'" @click="increment">
      Clicked {{ count }} times
    </button>
    <p v-if="count > 3 and not done">Hot!</p>
    <p v-else>Keep clicking</p>
    <input v-model="name" />
    <p>{{ f"Hello, {name}! {len(items)} steps." }}</p>
    <ul>
      <li v-for="item in items">{{ item.upper() }}</li>
    </ul>
  </section>
</template>
