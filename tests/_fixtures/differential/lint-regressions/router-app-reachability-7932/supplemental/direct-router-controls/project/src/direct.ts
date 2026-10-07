import { router } from "./router";
router.push({ name: "user", params: { id: "1" } });
router.push({ name: "absent-destination" });
router.push({ name: "user" });
router.push({ name: "user", params: { id: "1", extra: "x" } });
router.push({ name: "user", params: { id: [] } });
