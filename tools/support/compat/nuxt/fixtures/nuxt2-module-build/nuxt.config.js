export default {
  modules: ["@vizejs/nuxt"],
  build: { loaders: { imgUrl: { limit: 0 } } },
  vize: {
    lint: { autoInit: false },
    checker: { include: ["components/**/*.vue"] },
    musea: false,
  },
};
