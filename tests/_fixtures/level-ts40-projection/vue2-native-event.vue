<script lang="ts">
import Vue from "vue"
export default Vue.extend({
  props: { message: String },
})
</script>
<template><button @click.native="$emit('input', message)">{{ message }}</button></template>
