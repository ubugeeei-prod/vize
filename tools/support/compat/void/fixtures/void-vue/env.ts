import { defineEnv, string } from "void/env";

export default defineEnv({
  VITE_APP_NAME: string().default("Void with Vize"),
});
