export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("logger", {
    info(message: string) {
      return message.length;
    },
  });
};
