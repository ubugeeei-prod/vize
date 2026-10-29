import { inject } from "vue";

export const RequiredKey = Symbol("required");

export function useRequired(): void {
  inject(RequiredKey);
}
