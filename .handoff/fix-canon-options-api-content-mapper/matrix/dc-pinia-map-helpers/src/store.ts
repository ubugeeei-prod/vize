import { defineStore } from 'pinia'
export const useCounter = defineStore('counter', {
  state: () => ({ count: 0, name: 'c' }),
  getters: { double: (s) => s.count * 2 },
  actions: { increment(by: number) { this.count += by } },
})
