<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Test" component="./Foo.vue">
  <variant name="Default" default>
    <Foo />
  </variant>
</art>