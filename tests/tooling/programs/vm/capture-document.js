let records = fixture;
class DocumentNode {
  constructor(index) {
    this.index = index;
  }
  get nodeType() {
    return records[this.index].type;
  }
  get nodeValue() {
    return records[this.index].source;
  }
}
class Element extends DocumentNode {
  get outerHTML() {
    return records[this.index].source;
  }
  getBoundingClientRect() {
    return rect(records[this.index].bounds);
  }
  getClientRects() {
    return records[this.index].rectangles.map(rect);
  }
}
const rect = ([x, y, width, height]) => ({ x, y, width, height });
const makeNode = (index) => new (records[index].type === 1 ? Element : DocumentNode)(index);
let nodes = records.map((_, index) => makeNode(index));
globalThis.Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
globalThis.NodeFilter = { SHOW_ALL: -1 };
globalThis.document = {
  querySelector: () => nodes[0],
  createTreeWalker: () => {
    let index = 1;
    return { nextNode: () => nodes[index++] ?? null };
  },
  createRange: () => {
    let node;
    return {
      selectNodeContents: (value) => {
        node = value;
      },
      getBoundingClientRect: () => rect(records[node.index].bounds),
      getClientRects: () => records[node.index].rectangles.map(rect),
    };
  },
};

globalThis.captureDocument = {
  replaceRecords(next, replaceNode) {
    records = next;
    nodes = records.map((_, index) => nodes[index] ?? makeNode(index));
    if (replaceNode) nodes[1] = makeNode(1);
  },
  installInheritedToJSON() {
    let roots = 0;
    Array.prototype.toJSON = function (key) {
      return key === "" && ++roots > 2 ? [...this, roots] : this;
    };
  },
  nativeStringify: JSON.stringify,
  recordStringify(callback) {
    JSON.stringify = callback;
  },
  mutateDocument() {
    records[0].source = "changed during await";
    records[1].bounds[2] += 1;
  },
};
