export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("accountRepository", {
    find(id: string) {
      return { id, label: id.toUpperCase() };
    },
  });
};
