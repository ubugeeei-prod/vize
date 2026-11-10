// Generated from the canonical template-ref registry helpers. Do not edit.
// Template-ref registry targets, shared so each instantiation is cached once per program.
export type $DomElement<_Tag extends string, _Svg extends boolean = false> = _Svg extends true
  ? _Tag extends keyof SVGElementTagNameMap
    ? SVGElementTagNameMap[_Tag]
    : Element
  : _Tag extends keyof HTMLElementTagNameMap
    ? HTMLElementTagNameMap[_Tag]
    : Element;
export type $TemplateComponentRef<_C> = _C extends string
  ? _C extends keyof HTMLElementTagNameMap
    ? HTMLElementTagNameMap[_C]
    : _C extends keyof SVGElementTagNameMap
      ? SVGElementTagNameMap[_C]
      : Element
  : _C extends abstract new (...args: any[]) => infer _I
    ? _I
    : _C extends (props: any, ctx: any, expose: (exposed: infer _E) => any, ...args: any[]) => any
      ? NonNullable<_E>
      : any;
export type $RefInstanceFactory<_C> = 0 extends 1 & _C
  ? (component: _C) => (props: any) => undefined
  : _C extends { __vizeResolveRef?: infer _F }
    ? _F extends (...args: any[]) => any
      ? (component: _C) => _F
      : (component: _C) => (props: any) => undefined
    : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any
      ? (component: _C) => _C
      : (component: _C) => (props: any) => undefined;
export type $TemplateRefInstance<_T, _K extends keyof _T, _C> = 0 extends 1 & _C
  ? $TemplateComponentRef<_C>
  : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any
    ? _T[_K] extends () => infer _R
      ? _R extends { __ctx?: infer _X }
        ? NonNullable<_X> extends { expose: (exposed: infer _E) => any }
          ? NonNullable<_E>
          : $TemplateComponentRef<_C>
        : $TemplateComponentRef<_C>
      : $TemplateComponentRef<_C>
    : $TemplateComponentRef<_C>;
