import { createContext } from "../../foundations/context/context.ts";
import type { FocusVisibleState } from "./focus-visible-types.ts";

/** Focus-visible state published by FocusVisibleProvider. */
export const focusVisibleContext = createContext<FocusVisibleState>("FocusVisible");
