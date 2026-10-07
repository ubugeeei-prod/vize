import { createMemoryHistory, createRouter } from "vue-router";

// A throwaway router for a component test. It is not the app's route table.
export const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ path: "/", component: { template: "<div />" } }],
});
