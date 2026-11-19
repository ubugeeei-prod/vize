import { router } from "./router";

export function installReports() {
  router.addRoute({ path: "/reports/:year", name: "reports" });
}
