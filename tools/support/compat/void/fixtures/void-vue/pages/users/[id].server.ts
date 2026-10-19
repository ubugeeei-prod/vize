import { defineHandler, type InferProps } from "void";
import { USERS } from "./_data.ts";

export const loader = defineHandler((c) => {
  const user = USERS.find((candidate) => candidate.id === Number(c.req.param("id")));
  if (!user) return c.notFound();
  return { user };
});

export type Props = InferProps<typeof loader>;
