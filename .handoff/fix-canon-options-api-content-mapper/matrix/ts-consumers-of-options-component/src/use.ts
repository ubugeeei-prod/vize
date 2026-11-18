import { createApp, type ComponentPublicInstance } from 'vue'
import Child from './Child.vue'
type I = InstanceType<typeof Child>
declare const inst: I
const t: string = inst.title
const s: number = inst.size
const o: boolean = inst.toggle(true)
const bad: number = inst.title
inst.toggle('x')
inst.$emit('pick', 1)
inst.$emit('pick', 'x')
const p: I['$props'] = { title: 'a' }
const p2: I['$props'] = { title: 1 }
createApp(Child, { title: 'x' })
export const used: unknown[] = [t, s, o, bad, p, p2, inst as ComponentPublicInstance]
