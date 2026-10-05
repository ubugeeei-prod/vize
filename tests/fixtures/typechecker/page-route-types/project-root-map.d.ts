import "vue-router/auto-routes";

// A control for Vize's documented project-root fallback, not upstream output.
declare module "vue-router/auto-routes" {
  interface _RouteFileInfoMap {
    "packages/playground-file-based/src/pages/users/[userId=int].vue": {
      routes: "/users/[userId=int]";
      views: never;
      pathParamNames: "userId";
    };
  }
}
