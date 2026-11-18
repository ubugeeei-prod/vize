<script>
export default {
  props: { msg: String },
  data() { return { n: 1 } },
}
</script>

<template><div>{{ msg.toFixed() }} {{ n.toUpperCase() }} {{ nope }}</div></template>
