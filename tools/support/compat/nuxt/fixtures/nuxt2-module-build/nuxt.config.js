export default {
  modules: ["@vizejs/nuxt"],
  build: { loaders: { imgUrl: { limit: 0 } } },
  vize: {
    lint: false,
    musea: false,
  },
};
