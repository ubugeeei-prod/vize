<template>
  <main v-if="mode === 'a'">A</main>
  <main v-else-if="mode === 'b'">B</main>
  <main v-else>C</main>
  <header v-if="ok">A</header>
  <header v-else>B</header>
</template>
