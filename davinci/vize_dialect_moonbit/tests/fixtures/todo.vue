<script setup lang="moonbit">
struct Todo {
  title : String
  done : Bool
}

let todos : Array[Todo] = [
  { title: "Write the dialect", done: true },
  { title: "Give the talk", done: false },
]

let show_done : Ref[Bool] = { val: true }

fn remaining() -> Int {
  todos.filter(fn(todo) { !todo.done }).length()
}
</script>

<template>
  <h1>{{ remaining() }} left</h1>
  <button @click="show_done.val = !show_done.val">Toggle</button>
  <ul>
    <li v-for="todo in todos" v-show="show_done.val || !todo.done" :class="todo.done">
      {{ todo.title }}
    </li>
  </ul>
  <p v-if="remaining() > 0">Keep going</p>
  <p v-else>All done</p>
</template>
