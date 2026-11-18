import Child from './Child.vue'
declare module 'vue' {
  interface GlobalComponents { GlobalChild: typeof Child }
}
