import { defineComponent } from 'vue'
export const counter = defineComponent({
  data() { return { count: 0 } },
  computed: { double(): number { return this.count * 2 } },
  methods: { increment() { this.count++ } },
})
export const named = defineComponent({
  props: { label: { type: String, required: true } },
  methods: { greet(who: string) { return this.label + who } },
})
export const base = {
  data() { return { baseValue: 'b' } },
  methods: { baseMethod(n: number) { return n } },
}
