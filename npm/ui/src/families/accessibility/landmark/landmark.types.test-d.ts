/** Compile-only assertions for the public Landmark contract. */

import type {
  LandmarkElementMap,
  LandmarkInfo,
  LandmarkKeyBinding,
  LandmarkNavigationController,
  LandmarkRole,
  NamedLandmarkRole,
} from "./landmark.ts";
import {
  Landmark,
  LandmarkProvider,
  createLandmarkNavigation,
  isNamedLandmarkRole,
  landmarkElements,
  landmarkRoleOf,
} from "./landmark.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const controller: LandmarkNavigationController;
declare const info: LandmarkInfo;
declare const role: LandmarkRole;

type _Roles = Expect<
  Equal<
    LandmarkRole,
    | "banner"
    | "complementary"
    | "contentinfo"
    | "form"
    | "main"
    | "navigation"
    | "region"
    | "search"
  >
>;
type _Named = Expect<
  Equal<NamedLandmarkRole, "complementary" | "form" | "navigation" | "region" | "search">
>;
type _Main = Expect<Equal<LandmarkElementMap["main"], "main">>;
type _Region = Expect<Equal<(typeof landmarkElements)["region"], "section">>;
type _Element = Expect<Equal<typeof info.element, HTMLElement>>;
type _Label = Expect<Equal<typeof info.label, string | null>>;
type _Next = Expect<Equal<ReturnType<typeof controller.focusNext>, LandmarkInfo | null>>;
type _RoleOf = Expect<Equal<ReturnType<typeof landmarkRoleOf>, LandmarkRole | null>>;

if (isNamedLandmarkRole(role)) {
  type _Narrowed = Expect<Equal<typeof role, NamedLandmarkRole>>;
}

const binding: LandmarkKeyBinding = { key: "F6", shiftKey: true };
createLandmarkNavigation({ discover: true, nextKey: binding, previousKey: null });

const landmarkProps: InstanceType<typeof Landmark>["$props"] = {
  ariaLabel: "Primary",
  role: "navigation",
};
const providerProps: InstanceType<typeof LandmarkProvider>["$props"] = {
  disabled: false,
  discover: true,
  nextKey: { key: "F7", altKey: true },
  onNavigate: (landmark: LandmarkInfo) => landmark.role,
};

// @ts-expect-error role is required.
const missingRole: InstanceType<typeof Landmark>["$props"] = { ariaLabel: "Nav" };

// @ts-expect-error role must be a landmark role.
const badRole: InstanceType<typeof Landmark>["$props"] = { role: "button" };

// @ts-expect-error key bindings need a key.
const badBinding: LandmarkKeyBinding = { shiftKey: true };

void badBinding;
void badRole;
void landmarkProps;
void missingRole;
void providerProps;
