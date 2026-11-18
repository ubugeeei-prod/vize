<script lang="ts">
import { defineComponent } from 'vue'
import { mapGetters, mapState, mapActions, mapMutations } from 'vuex'
interface State { count: number; user: { name: string } }
export default defineComponent({
  computed: {
    ...mapGetters(['total', 'isReady']),
    ...mapState({ count: (s: any) => (s as State).count }),
    ...mapState(['user']),
  },
  methods: { ...mapActions(['load']), ...mapMutations({ setCount: 'SET_COUNT' }) },
})
</script>

<template>
  <div>
    {{ total.anything }} {{ isReady }} {{ count }} {{ user.name }}
    <button @click="load(); setCount(1)">x</button>
  </div>
</template>
