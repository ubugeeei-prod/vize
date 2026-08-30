import { createContext } from "../../../context.ts";

import type { PositionerController } from "./positioner-types.ts";

/** Compound-component context shared by Positioner and PositionerArrow. */
export const positionerContext = createContext<PositionerController>("Positioner");
