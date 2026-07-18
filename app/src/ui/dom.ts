/** Small DOM construction helpers shared by the ui components. */

export type ElOptions = {
  readonly className?: string;
  readonly text?: string;
  readonly attrs?: Readonly<Record<string, string>>;
};

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  options: ElOptions = {},
  children: readonly (HTMLElement | string)[] = [],
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  if (options.className !== undefined) {
    element.className = options.className;
  }
  if (options.text !== undefined) {
    element.textContent = options.text;
  }
  if (options.attrs !== undefined) {
    for (const [name, value] of Object.entries(options.attrs)) {
      element.setAttribute(name, value);
    }
  }
  element.append(...children);
  return element;
}

let nextUiId = 0;

/** Returns a document-unique id for ARIA wiring. */
export function uiId(prefix: string): string {
  nextUiId += 1;
  return `${prefix}-${nextUiId}`;
}
