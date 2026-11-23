import { requiredEnv } from "../shared/files.ts";
import fs from "node:fs";
fs.mkdirSync(requiredEnv("VIZE_PRIVATE_OVERLAY_CAPTURE"), { mode: 0o700 });
