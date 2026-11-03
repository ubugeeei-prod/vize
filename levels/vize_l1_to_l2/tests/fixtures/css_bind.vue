<template>
<p class="foo">{{ color }}</p>
</template>
<style>
.foo { color: v-bind(color); }
</style>
