export interface FocusScopeToken {
  document: Document | null;
  root: Element | null;
  readonly readContain: () => boolean;
  lastFocused: HTMLElement | null;
}

const documentScopes = new WeakMap<Document, FocusScopeToken[]>();

function scopesFor(document: Document): FocusScopeToken[] {
  let scopes = documentScopes.get(document);
  if (!scopes) {
    scopes = [];
    documentScopes.set(document, scopes);
  }
  return scopes;
}

export function attachScope(token: FocusScopeToken, root: Element): void {
  if (token.document === root.ownerDocument) {
    token.root = root;
    return;
  }
  detachScope(token);
  token.document = root.ownerDocument;
  token.root = root;
  scopesFor(root.ownerDocument).push(token);
}

export function detachScope(token: FocusScopeToken): void {
  const document = token.document;
  if (document) {
    const scopes = scopesFor(document);
    const index = scopes.indexOf(token);
    if (index >= 0) scopes.splice(index, 1);
  }
  token.document = null;
  token.root = null;
}

export function containmentOwner(document: Document): FocusScopeToken | null {
  const scopes = scopesFor(document);
  for (let index = scopes.length - 1; index >= 0; index--) {
    const token = scopes[index];
    try {
      if (token?.readContain()) return token;
    } catch {
      // A scope with an unreadable contain option cannot own containment, and must not
      // hide the scopes below it from the shared document listeners.
    }
  }
  return null;
}

export function rootsOwnedBy(token: FocusScopeToken): Element[] {
  const document = token.document;
  if (!document) return [];
  const scopes = scopesFor(document);
  const index = scopes.indexOf(token);
  if (index < 0) return [];
  return scopes.slice(index).flatMap(({ root }) => (root ? [root] : []));
}

export function parentScope(token: FocusScopeToken): FocusScopeToken | null {
  const document = token.document;
  if (!document) return null;
  const scopes = scopesFor(document);
  const index = scopes.indexOf(token);
  return index > 0 ? (scopes[index - 1] ?? null) : null;
}
