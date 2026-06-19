declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}

declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn extends Omit<
    import("@nuxt/types").Context,
    "route" | "query" | "from" | "params"
  > {}
  export function useContext(): UseContextReturn;
}

declare module "#app" {
  export interface NuxtApp {}
}
