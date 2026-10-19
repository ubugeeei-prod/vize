import { defineHandler, type InferProps } from "void";
import { env } from "void/env";

export const loader = defineHandler(() => {
  return { title: env.VITE_APP_NAME, count: 2 };
});

export type Props = InferProps<typeof loader>;
