import { defineConfig } from "tsdown";
import vitePlusConfig from "./vite.config.ts";
import { getTsdownPackConfig } from "../../tools/tsdown/from-vite-plus.ts";

export default defineConfig(getTsdownPackConfig(vitePlusConfig));
