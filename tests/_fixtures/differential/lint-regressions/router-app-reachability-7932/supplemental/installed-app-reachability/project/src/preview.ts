import { useRouter } from "vue-router";
export interface PreviewState { value: string }
const router = useRouter();
router.push({ name: "preview-only" });
