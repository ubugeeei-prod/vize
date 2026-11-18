import { h, type FunctionalComponent } from 'vue'
export const Fn: FunctionalComponent<{ msg: string }, { pick: (n: number) => void }> = (props, { emit }) =>
  h('div', { onClick: () => emit('pick', 1) }, props.msg)
Fn.props = ['msg']
Fn.emits = ['pick']
