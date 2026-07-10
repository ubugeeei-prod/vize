export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("auth", {
    loggedIn: true,
    userName() {
      return "Ada";
    },
  });
};
