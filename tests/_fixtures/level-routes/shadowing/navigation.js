import appRouter from "./router.js";
import { useRouter } from "./fake-router.js";

// A parameter named `router` is not the router.
export function go(router) {
  router.push({ name: "nope" });
}

// `useRouter` from another module is not Vue Router's.
export function goFake() {
  const router = useRouter();
  router.push({ name: "nope" });
}

// A local `routes` shadows nothing: only the router's own records count.
export function local() {
  const routes = [{ name: "nope" }];
  return routes;
}

// The real router, imported by default export: this one is checked.
export function goReal() {
  appRouter.push({ name: "hmoe" });
}
