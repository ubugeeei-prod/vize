// The original typed-slot capture host, with complete calls and function identities.
export function typedSlotHost(record, snapshot) {
  const functions = new WeakMap();
  let nextFunction = 1;
  const functionId = (fn) => {
    if (!functions.has(fn)) functions.set(fn, nextFunction++);
    return functions.get(fn);
  };
  const hostValue = (value) =>
    typeof value === "function"
      ? { type: "function", id: functionId(value) }
      : value === undefined
        ? { $undefined: true }
        : snapshot(value);
  const make = (type, text = null) => ({ type, props: {}, text, children: [], parent: null });
  const freeze = (node, semantic = false) => ({
    type: node.type,
    props: Object.fromEntries(
      Object.entries(node.props).map(([k, v]) => [
        k,
        typeof v === "function" && semantic ? "function" : hostValue(v),
      ]),
    ),
    text: node.text,
    children: node.children.map((n) => freeze(n, semantic)),
  });
  const host = {
    createElement: (type) => {
      record.hostCalls.push({ method: "createElement", type });
      return make(type);
    },
    createText: (text) => {
      record.hostCalls.push({ method: "createText", text });
      return make("#text", text);
    },
    createComment: (text) => {
      record.hostCalls.push({ method: "createComment", text });
      return make("#comment", text);
    },
    setText: (node, text) => {
      record.hostCalls.push({ method: "setText", type: node.type, text });
      node.text = text;
    },
    setElementText: (node, text) => {
      record.hostCalls.push({ method: "setElementText", type: node.type, text });
      node.text = text;
      node.children = [];
    },
    patchProp: (node, key, previous, next) => {
      record.hostCalls.push({
        method: "patchProp",
        type: node.type,
        key,
        previous: hostValue(previous),
        next: hostValue(next),
      });
      node.props[key] = next;
    },
    insert: (node, parent, anchor = null) => {
      record.hostCalls.push({
        method: "insert",
        type: node.type,
        parentType: parent.type,
        anchorType: anchor?.type ?? null,
      });
      if (node.parent) {
        const old = node.parent.children;
        old.splice(old.indexOf(node), 1);
      }
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
      node.parent = parent;
    },
    remove: (node) => {
      record.hostCalls.push({ method: "remove", type: node.type });
      if (node.parent) node.parent.children.splice(node.parent.children.indexOf(node), 1);
      node.parent = null;
    },
    parentNode: (node) => node.parent,
    nextSibling: (node) => node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  };
  const find = (node, id) =>
    node.props.id === id ? node : node.children.map((n) => find(n, id)).find(Boolean);
  return { host, make, freeze, find, functionId, hostValue };
}
