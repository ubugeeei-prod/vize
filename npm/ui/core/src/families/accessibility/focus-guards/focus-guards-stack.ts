export interface FocusGuardsToken {
  document: Document | null;
  root: Element | null;
  readonly readEnabled: () => boolean;
  readonly setTopmost: (value: boolean) => void;
}

const documentStacks = new WeakMap<Document, FocusGuardsToken[]>();

function stackFor(document: Document): FocusGuardsToken[] {
  let stack = documentStacks.get(document);
  if (!stack) {
    stack = [];
    documentStacks.set(document, stack);
  }
  return stack;
}

export function recomputeFocusGuards(document: Document): void {
  const stack = stackFor(document);
  let owner: FocusGuardsToken | null = null;
  for (let index = stack.length - 1; index >= 0; index--) {
    const token = stack[index];
    if (token?.root?.isConnected && token.readEnabled()) {
      owner = token;
      break;
    }
  }
  for (const token of stack) token.setTopmost(token === owner);
}

export function attachFocusGuards(token: FocusGuardsToken, root: Element): void {
  if (token.document === root.ownerDocument) {
    token.root = root;
    recomputeFocusGuards(root.ownerDocument);
    return;
  }
  detachFocusGuards(token);
  token.document = root.ownerDocument;
  token.root = root;
  stackFor(root.ownerDocument).push(token);
  recomputeFocusGuards(root.ownerDocument);
}

export function detachFocusGuards(token: FocusGuardsToken): void {
  const document = token.document;
  if (!document) {
    token.root = null;
    token.setTopmost(false);
    return;
  }
  const stack = stackFor(document);
  const index = stack.indexOf(token);
  if (index >= 0) stack.splice(index, 1);
  token.document = null;
  token.root = null;
  token.setTopmost(false);
  if (stack.length === 0) documentStacks.delete(document);
  else recomputeFocusGuards(document);
}
