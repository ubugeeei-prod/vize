import { reactive } from "vue";
const state = reactive({ count: 0 });
const { count } = state;
void count;
