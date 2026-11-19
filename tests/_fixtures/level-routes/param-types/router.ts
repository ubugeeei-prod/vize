import * as VueRouter from "vue-router";

const routes = [
  { path: "/items/:id", name: "item" },
  { path: "/files/:segments+", name: "files" },
  { path: "/docs/:rest*", name: "docs" },
  { path: "/find/:q?", name: "find" },
];

export default VueRouter.createRouter({ history: VueRouter.createMemoryHistory(), routes });
