import { defineHandler, type InferProps } from "void";
import { USERS } from "./_data.ts";

export const loader = defineHandler(() => ({ users: [...USERS] }));

export type Props = InferProps<typeof loader>;
