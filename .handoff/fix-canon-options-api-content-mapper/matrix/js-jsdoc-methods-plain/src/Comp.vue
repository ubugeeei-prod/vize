<script>
/** @typedef {{ id: number, name: string }} Row */
export default {
  props: {
    /** @type {import('vue').PropType<Row[]>} */
    rows: { type: Array, required: true },
  },
  data() {
    return { /** @type {string | null} */ selected: null }
  },
  computed: {
    /** @returns {number} */
    total() { return this.rows.length },
  },
  methods: {
    /** @param {Row} row */
    pick(row) { this.selected = row.name },
    /**
     * @param {number} a
     * @param {number} b
     */
    add(a, b) { return a + b },
  },
}
</script>

<template>
  <div>
    <span v-for="r in rows" :key="r.id" @click="pick(r)">{{ r.name }}</span>
    {{ total.toFixed() }} {{ add(1, 2) }} {{ add('x', 2) }} {{ selected?.length }}
  </div>
</template>
