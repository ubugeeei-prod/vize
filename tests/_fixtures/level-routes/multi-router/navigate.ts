import { useRouter } from "vue-router";
import adminRouter from "./admin-router";
import { siteRouter } from "./site-router";

export function useNavigation() {
  const router = useRouter();
  // Declared by both routers with different params: no param claim.
  router.push({ name: "dashboard", params: { team: "core" } });
  // Declared by neither router: proven unknown against both trees.
  router.push({ name: "settings" });
  // Declared once: its params are checked whichever app runs this.
  router.push({ name: "member", params: { id: 7 } });
}

// A specific router only reaches its own names.
adminRouter.push({ name: "pricing" });
siteRouter.push({ name: "pricing" });
