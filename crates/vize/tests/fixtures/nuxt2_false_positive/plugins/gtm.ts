export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("gtm", {
    push(event: { name: string }) {
      return event.name;
    },
  });
};
