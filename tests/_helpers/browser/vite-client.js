const noop = () => {};
const hotContext = {
  accept: noop,
  acceptExports: noop,
  data: {},
  decline: noop,
  dispose: noop,
  invalidate: noop,
  off: noop,
  on: noop,
  prune: noop,
  send: noop,
};
export const createHotContext = () => hotContext;
export const injectQuery = (url) => url;
export const removeStyle = noop;
export const updateStyle = noop;
export default {};
