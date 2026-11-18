import { defineComponent, h } from 'vue'
export const Gen = defineComponent(
  <T extends string | number>(props: { item: T; list: T[] }) => () => h('div', String(props.item)),
  { props: ['item', 'list'] },
)
