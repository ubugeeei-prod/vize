<script setup lang="moonbit">
@vue.define_props(["title"])

let count = @vue.ref(0)
let done = @vue.ref(false)
let name = @vue.ref("world")
let items = @vue.ref(["write", "compile", "ship"])

///| Clicking past five marks the counter done.
fn increment() -> Unit {
  count.val += 1
  done.val = count.val > 5
}

fn tone(count : Int) -> String {
  if count > 3 { "hot" } else { "cold" }
}
</script>

<template>
  <section class="counter">
    <h1>{{ title }}</h1>
    <button type="button" :class="count |> tone" @click="increment">
      Clicked {{ count }} times
    </button>
    <p v-if="count > 3 && not(done)">Hot!</p>
    <p v-else>Keep clicking</p>
    <input v-model="name" />
    <p>{{ "Hello, \{name}!" }}</p>
    <ul>
      <li v-for="item in items">{{ item.to_upper() }}</li>
    </ul>
  </section>
</template>
