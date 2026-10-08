import type { ContractExample } from "./types.ts";
import { contract0Examples } from "./project-contracts-0.ts";
import { contract1Examples } from "./project-contracts-1.ts";
import { contract2Examples } from "./project-contracts-2.ts";
import { contract3Examples } from "./project-contracts-3.ts";

export const contractExamples: Record<string, ContractExample> = {
  ...contract0Examples,
  ...contract1Examples,
  ...contract2Examples,
  ...contract3Examples,
};
