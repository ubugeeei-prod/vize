import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

const implementation = fs.readFileSync(
  new URL(
    "../../crates/vize_canon/src/virtual_ts/scope/component_events/inference_helpers.rs",
    import.meta.url,
  ),
  "utf8",
);
const declaration = implementation.match(/const EMIT_INFERENCE_HELPERS: &str = r#"([\s\S]+?)"#;/);
assert.ok(declaration);
const helper = declaration[1];

// These hand-authored constructor/input models isolate this helper boundary.
// Genuine Vue, emitted modules, native mappings and original upstream packets
// are independently exercised by the native/full-tooling/official campaigns.
// The canonical schema models below retain unknown-prop policy; the secondary
// helper must validate typed inherited values without inventing excess checks.
const body = `declare const constructor_0: (new(props:{own?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra?:string};};
declare const factory_0: __VizeEmitPropsFactory<typeof constructor_0>;
factory_0(constructor_0)({own:1,extra:"ok"});
declare const constructor_1: (new(props:{own?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra?:string};};
declare const factory_1: __VizeEmitPropsFactory<typeof constructor_1>;
factory_1(constructor_1)({own:1,extra:123});
declare const constructor_2: (new(props:{own?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra?:string};};
declare const factory_2: __VizeEmitPropsFactory<typeof constructor_2>;
factory_2(constructor_2)({...{own:1,extra:"ok"}});
declare const constructor_3: (new(props:{own?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra?:string};};
declare const factory_3: __VizeEmitPropsFactory<typeof constructor_3>;
factory_3(constructor_3)({...{own:1,extra:123}});
declare const constructor_4: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:Record<string,string>;};
declare const factory_4: __VizeEmitPropsFactory<typeof constructor_4>;
factory_4(constructor_4)({bar:1,extra:"ok"});
declare const constructor_5: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:Record<string,string>;};
declare const factory_5: __VizeEmitPropsFactory<typeof constructor_5>;
factory_5(constructor_5)({bar:1,extra:123});
declare const constructor_6: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:Record<string,string>;};
declare const factory_6: __VizeEmitPropsFactory<typeof constructor_6>;
factory_6(constructor_6)({bar:"wrong",extra:"ok"});
declare const constructor_7: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:Record<string,string>;};
declare const factory_7: __VizeEmitPropsFactory<typeof constructor_7>;
factory_7(constructor_7)({...{bar:1,extra:"ok"}});
declare const constructor_8: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:Record<string,string>;};
declare const factory_8: __VizeEmitPropsFactory<typeof constructor_8>;
factory_8(constructor_8)({...{bar:1,extra:123}});
declare const constructor_9: (new(props:{'data-owned'?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{'data-owned'?:number},Exclude<'data-owned','hidden'|'onChange'>>;__vizeEmitProps?:Select<{'data-owned'?:number},Extract<'data-owned','onChange'>>;__vizeFallthroughProps?:Partial<Record<\`data-\${string}\`,string>>;};
declare const factory_9: __VizeEmitPropsFactory<typeof constructor_9>;
factory_9(constructor_9)({"data-owned":1,"data-extra":"ok"});
declare const constructor_10: (new(props:{'data-owned'?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{'data-owned'?:number},Exclude<'data-owned','hidden'|'onChange'>>;__vizeEmitProps?:Select<{'data-owned'?:number},Extract<'data-owned','onChange'>>;__vizeFallthroughProps?:Partial<Record<\`data-\${string}\`,string>>;};
declare const factory_10: __VizeEmitPropsFactory<typeof constructor_10>;
factory_10(constructor_10)({"data-owned":1,"data-extra":123});
declare const constructor_11: (new(props:{'data-owned'?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{'data-owned'?:number},Exclude<'data-owned','hidden'|'onChange'>>;__vizeEmitProps?:Select<{'data-owned'?:number},Extract<'data-owned','onChange'>>;__vizeFallthroughProps?:Partial<Record<\`data-\${string}\`,string>>;};
declare const factory_11: __VizeEmitPropsFactory<typeof constructor_11>;
factory_11(constructor_11)({"data-owned":"wrong","data-extra":"ok"});
declare const constructor_12: (new(props:{'data-owned'?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{'data-owned'?:number},Exclude<'data-owned','hidden'|'onChange'>>;__vizeEmitProps?:Select<{'data-owned'?:number},Extract<'data-owned','onChange'>>;__vizeFallthroughProps?:Partial<Record<\`data-\${string}\`,string>>;};
declare const factory_12: __VizeEmitPropsFactory<typeof constructor_12>;
factory_12(constructor_12)({...{"data-owned":1,"data-extra":"ok"}});
declare const constructor_13: (new(props:{'data-owned'?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{'data-owned'?:number},Exclude<'data-owned','hidden'|'onChange'>>;__vizeEmitProps?:Select<{'data-owned'?:number},Extract<'data-owned','onChange'>>;__vizeFallthroughProps?:Partial<Record<\`data-\${string}\`,string>>;};
declare const factory_13: __VizeEmitPropsFactory<typeof constructor_13>;
factory_13(constructor_13)({...{"data-owned":1,"data-extra":123}});
declare const constructor_14: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:{bar?:string};};
declare const factory_14: __VizeEmitPropsFactory<typeof constructor_14>;
factory_14(constructor_14)({bar:1});
declare const constructor_15: (new(props:{bar?:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{bar?:number},Exclude<'bar','hidden'|'onChange'>>;__vizeEmitProps?:Select<{bar?:number},Extract<'bar','onChange'>>;__vizeFallthroughProps?:{bar?:string};};
declare const factory_15: __VizeEmitPropsFactory<typeof constructor_15>;
factory_15(constructor_15)({bar:"wrong"});
declare const constructor_16: (new(props:{hidden?:boolean}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{hidden?:boolean},Exclude<'hidden','hidden'|'onChange'>>;__vizeEmitProps?:Select<{hidden?:boolean},Extract<'hidden','onChange'>>;__vizeFallthroughProps?:{hidden?:string};};
declare const factory_16: __VizeEmitPropsFactory<typeof constructor_16>;
factory_16(constructor_16)({hidden:true});
declare const constructor_17: (new(props:{hidden?:boolean}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{hidden?:boolean},Exclude<'hidden','hidden'|'onChange'>>;__vizeEmitProps?:Select<{hidden?:boolean},Extract<'hidden','onChange'>>;__vizeFallthroughProps?:{hidden?:string};};
declare const factory_17: __VizeEmitPropsFactory<typeof constructor_17>;
factory_17(constructor_17)({hidden:"wrong"});
declare const constructor_18: (new(props:{own?:number;onClick?:(value:MouseEvent)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Extract<'own','onChange'>>;__vizeFallthroughProps?:{onClick?:(value:string)=>void};};
declare const factory_18: __VizeEmitPropsFactory<typeof constructor_18>;
factory_18(constructor_18)({own:1,onClick:(value:string)=>{}});
declare const constructor_19: (new(props:{own?:number;onClick?:(value:MouseEvent)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Extract<'own','onChange'>>;__vizeFallthroughProps?:{onClick?:(value:string)=>void};};
declare const factory_19: __VizeEmitPropsFactory<typeof constructor_19>;
factory_19(constructor_19)({own:1,onClick:(value:MouseEvent)=>{}});
declare const constructor_20: (new(props:{own?:number;onClick?:(value:MouseEvent)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Extract<'own','onChange'>>;__vizeFallthroughProps?:{onClick?:(value:string)=>void};};
declare const factory_20: __VizeEmitPropsFactory<typeof constructor_20>;
factory_20(constructor_20)({own:1,onClick:(value)=>{const text:string=value;}});
declare const constructor_21: (new(props:{own?:number;onClick?:(value:MouseEvent)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own?:number;onClick?:(value:MouseEvent)=>void},Extract<'own','onChange'>>;__vizeFallthroughProps?:{onClick?:(value:string)=>void};};
declare const factory_21: __VizeEmitPropsFactory<typeof constructor_21>;
factory_21(constructor_21)({own:1,onClick:(value)=>{const number:number=value;}});
declare const constructor_22: (new(props:{onChange?:(value:number)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{onChange?:(value:number)=>void},Exclude<'onChange','hidden'|'onChange'>>;__vizeEmitProps?:Select<{onChange?:(value:number)=>void},Extract<'onChange','onChange'>>;__vizeFallthroughProps?:{onChange?:(value:string)=>void};};
declare const factory_22: __VizeEmitPropsFactory<typeof constructor_22>;
factory_22(constructor_22)({onChange:(value:number)=>{}});
declare const constructor_23: (new(props:{onChange?:(value:number)=>void}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{onChange?:(value:number)=>void},Exclude<'onChange','hidden'|'onChange'>>;__vizeEmitProps?:Select<{onChange?:(value:number)=>void},Extract<'onChange','onChange'>>;__vizeFallthroughProps?:{onChange?:(value:string)=>void};};
declare const factory_23: __VizeEmitPropsFactory<typeof constructor_23>;
factory_23(constructor_23)({onChange:(value:string)=>{}});
declare const constructor_24: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra:string};};
declare const factory_24: __VizeEmitPropsFactory<typeof constructor_24>;
factory_24(constructor_24)({own:1,extra:"ok"});
declare const constructor_25: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra:string};};
declare const factory_25: __VizeEmitPropsFactory<typeof constructor_25>;
factory_25(constructor_25)({extra:"ok"});
declare const constructor_26: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra:string};};
declare const factory_26: __VizeEmitPropsFactory<typeof constructor_26>;
factory_26(constructor_26)({own:1});
declare const constructor_27: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{extra:string};};
declare const factory_27: __VizeEmitPropsFactory<typeof constructor_27>;
factory_27(constructor_27)({});
declare const constructor_28: (new(props:{kind:"left";left:string}|{kind:"right";right:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Exclude<'kind'|'left'|'right','hidden'|'onChange'>>;__vizeEmitProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Extract<'kind'|'left'|'right','onChange'>>;__vizeFallthroughProps?:{extra?:boolean};};
declare const factory_28: __VizeEmitPropsFactory<typeof constructor_28>;
factory_28(constructor_28)({kind:"left",left:"ok",extra:true});
declare const constructor_29: (new(props:{kind:"left";left:string}|{kind:"right";right:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Exclude<'kind'|'left'|'right','hidden'|'onChange'>>;__vizeEmitProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Extract<'kind'|'left'|'right','onChange'>>;__vizeFallthroughProps?:{extra?:boolean};};
declare const factory_29: __VizeEmitPropsFactory<typeof constructor_29>;
factory_29(constructor_29)({kind:"right",right:1,extra:true});
declare const constructor_30: (new(props:{kind:"left";left:string}|{kind:"right";right:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Exclude<'kind'|'left'|'right','hidden'|'onChange'>>;__vizeEmitProps?:Select<{kind:"left";left:string}|{kind:"right";right:number},Extract<'kind'|'left'|'right','onChange'>>;__vizeFallthroughProps?:{extra?:boolean};};
declare const factory_30: __VizeEmitPropsFactory<typeof constructor_30>;
factory_30(constructor_30)({kind:"left",left:1,extra:true});
declare const constructor_31: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{};};
declare const factory_31: __VizeEmitPropsFactory<typeof constructor_31>;
factory_31(constructor_31)({own:1});
declare const constructor_32: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{};};
declare const factory_32: __VizeEmitPropsFactory<typeof constructor_32>;
factory_32(constructor_32)({own:"wrong"});
declare const constructor_33: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:{};};
declare const factory_33: __VizeEmitPropsFactory<typeof constructor_33>;
factory_33(constructor_33)({});
declare const constructor_34: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:unknown;};
declare const factory_34: __VizeEmitPropsFactory<typeof constructor_34>;
factory_34(constructor_34)({own:1});
declare const constructor_35: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:unknown;};
declare const factory_35: __VizeEmitPropsFactory<typeof constructor_35>;
factory_35(constructor_35)({own:"wrong"});
declare const constructor_36: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:unknown;};
declare const factory_36: __VizeEmitPropsFactory<typeof constructor_36>;
factory_36(constructor_36)({});
declare const constructor_37: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:undefined;};
declare const factory_37: __VizeEmitPropsFactory<typeof constructor_37>;
factory_37(constructor_37)({own:1});
declare const constructor_38: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:undefined;};
declare const factory_38: __VizeEmitPropsFactory<typeof constructor_38>;
factory_38(constructor_38)({own:"wrong"});
declare const constructor_39: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:undefined;};
declare const factory_39: __VizeEmitPropsFactory<typeof constructor_39>;
factory_39(constructor_39)({});
declare const constructor_40: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:null;};
declare const factory_40: __VizeEmitPropsFactory<typeof constructor_40>;
factory_40(constructor_40)({own:1});
declare const constructor_41: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:null;};
declare const factory_41: __VizeEmitPropsFactory<typeof constructor_41>;
factory_41(constructor_41)({own:"wrong"});
declare const constructor_42: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:null;};
declare const factory_42: __VizeEmitPropsFactory<typeof constructor_42>;
factory_42(constructor_42)({});
declare const constructor_43: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:never;};
declare const factory_43: __VizeEmitPropsFactory<typeof constructor_43>;
factory_43(constructor_43)({own:1});
declare const constructor_44: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:never;};
declare const factory_44: __VizeEmitPropsFactory<typeof constructor_44>;
factory_44(constructor_44)({own:"wrong"});
declare const constructor_45: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:never;};
declare const factory_45: __VizeEmitPropsFactory<typeof constructor_45>;
factory_45(constructor_45)({});
declare const constructor_46: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:any;};
declare const factory_46: __VizeEmitPropsFactory<typeof constructor_46>;
factory_46(constructor_46)({own:1});
declare const constructor_47: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:any;};
declare const factory_47: __VizeEmitPropsFactory<typeof constructor_47>;
factory_47(constructor_47)({own:"wrong"});
declare const constructor_48: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;__vizeFallthroughProps?:any;};
declare const factory_48: __VizeEmitPropsFactory<typeof constructor_48>;
factory_48(constructor_48)({});
declare const constructor_49: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;};
declare const factory_49: __VizeEmitPropsFactory<typeof constructor_49>;
factory_49(constructor_49)({own:1});
declare const constructor_50: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;};
declare const factory_50: __VizeEmitPropsFactory<typeof constructor_50>;
factory_50(constructor_50)({own:"wrong"});
declare const constructor_51: (new(props:{own:number}) => {$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:Select<{own:number},Exclude<'own','hidden'|'onChange'>>;__vizeEmitProps?:Select<{own:number},Extract<'own','onChange'>>;};
declare const factory_51: __VizeEmitPropsFactory<typeof constructor_51>;
factory_51(constructor_51)({});
declare const generic: new<T>(props:{value:T}) => {$emit:(value:T)=>void};
declare const genericFactory: __VizeEmitPropsFactory<typeof generic>;
const emit = genericFactory(generic)({value:"text"}).emit;
emit("ok");
emit(123);
declare const generatedGeneric: {readonly __vizeComponentMarker:true;__vizeResolveProps?:<T>(props:{value:T;extra?:string})=>{emit:(value:T)=>void}};
declare const generatedGenericFactory: __VizeEmitPropsFactory<typeof generatedGeneric>;
const generatedEmit = generatedGenericFactory(generatedGeneric)({value:"text",extra:"root"}).emit;
generatedEmit("ok");
generatedEmit(123);
declare const internalGeneric: (new<A=unknown>(props?:A&{own?:number})=>{$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:{own?:number};__vizeFallthroughProps?:{extra?:string}};
declare const internalFactory: __VizeEmitPropsFactory<typeof internalGeneric>;
internalFactory(internalGeneric)({own:1,extra:"root"});
internalFactory(internalGeneric)({own:"wrong",extra:"root"});
type IsUnknown<T> = unknown extends T ? [keyof T] extends [never] ? true : false : false;
type PublicMode<A> = __VizeIsAny<A> extends true ? true : IsUnknown<A>;
type PublicInput<P> = P extends unknown ? Partial<P> & {hidden?:boolean;onClick?:(event:MouseEvent)=>void} : never;
type Missing<P,A> = Exclude<__VizeEmitRequiredKeys<P>,keyof A>;
type Guard<P,A> = [Missing<P,A>|Exclude<keyof A,keyof PublicInput<P>>] extends [never] ? {} : {readonly invalidProps:{readonly missing:Missing<P,A>;readonly extra:Exclude<keyof A,keyof PublicInput<P>>}};
type Input<P,A> = PublicMode<A> extends true ? PublicInput<P> : P extends unknown ? PublicInput<P>&Guard<P,A> : never;
type View<P> = {$props:P;$emit:(value:string)=>void};
declare const conditional: (new<A=unknown>(props?: A&Input<{own:number},A>)=>PublicMode<A> extends true ? View<{own:number}> : Omit<View<{own:number}>,"$props"> & {$props:{[K in keyof A]:unknown}&PublicInput<{own:number}>&Guard<{own:number},A>}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:{own:number};__vizeFallthroughProps?:Record<string,string>};
declare const conditionalFactory: __VizeEmitPropsFactory<typeof conditional>;
conditionalFactory(conditional)({own:1,extra:"ok"});
conditionalFactory(conditional)({own:"wrong",extra:"ok"});
conditionalFactory(conditional)({own:1,extra:123});
conditionalFactory(conditional)({...{own:1,extra:123}});
conditionalFactory(conditional)({own:1,anyOtherKey:"typed"});
conditionalFactory(conditional)({extra:"ok"});
declare function canonicalOwn(props:{own:number}):void;
canonicalOwn({});
declare const untypedMetadata: (new(props:{own?:number;hidden?:boolean})=>{$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:any;__vizeEmitProps?:any;__vizeFallthroughProps?:{extra?:string;hidden?:string}};
declare const untypedFactory: __VizeEmitPropsFactory<typeof untypedMetadata>;
untypedFactory(untypedMetadata)({own:1,hidden:true,extra:"root"});
untypedFactory(untypedMetadata)({own:1,hidden:true,extra:123});
untypedFactory(untypedMetadata)({own:1,hidden:"wrong",extra:"root"});
declare const schemaComponent: (new(props:{own?:number})=>{$emit:(value:string)=>void}) & {readonly __vizeComponentMarker:true;__vizeRawProps?:{own?:number};__vizeFallthroughProps?:{extra?:string}};
declare const schemaFactory: __VizeEmitPropsFactory<typeof schemaComponent>;
declare function schemaOpen(props:{own?:number;extra?:unknown}&Record<string,unknown>):void;
declare function schemaStrict(props:{own?:number;extra?:unknown}):void;
schemaOpen({own:1,extra:"ok",bogus:123});
schemaOpen({own:1,extra:"ok",bogus:123}); schemaFactory(schemaComponent)({own:1,extra:"ok",bogus:123});
schemaStrict({own:1,extra:"ok",bogus:123});
schemaStrict({own:1,extra:"ok",bogus:123}); schemaFactory(schemaComponent)({own:1,extra:"ok",bogus:123});
const canonicalIndexedValue:string=123;
const canonicalIndexedValue2:string=123; conditionalFactory(conditional)({own:1,extra:123});
`;
const cases: Array<[string, number, number[]]> = [
  ["finite-valid", 2, []],
  ["finite-wrong", 5, [2322]],
  ["finite-spread-valid", 8, []],
  ["finite-spread-wrong", 11, [2345]],
  ["broad-valid", 14, []],
  ["broad-wrong", 17, [2322]],
  ["broad-own-wrong", 20, [2322]],
  ["broad-spread-valid", 23, []],
  ["broad-spread-wrong", 26, [2345]],
  ["pattern-valid", 29, []],
  ["pattern-wrong", 32, [2322]],
  ["pattern-own-wrong", 35, [2322]],
  ["pattern-spread-valid", 38, []],
  ["pattern-spread-wrong", 41, [2345]],
  ["own-collision-valid", 44, []],
  ["own-collision-wrong", 47, [2322]],
  ["public-collision-valid", 50, []],
  ["public-collision-wrong", 53, [2322]],
  ["ambient-click-valid", 56, []],
  ["ambient-click-wrong", 59, [2322]],
  ["ambient-click-context-valid", 62, []],
  ["ambient-click-context-wrong", 65, [2322]],
  ["own-emit-valid", 68, []],
  ["own-emit-wrong", 71, [2322]],
  ["required-valid", 74, []],
  ["required-own-missing", 77, [2345]],
  ["required-inherited-missing", 80, [2345]],
  ["required-both-missing", 83, [2345]],
  ["union-left-valid", 86, []],
  ["union-right-valid", 89, []],
  ["union-wrong", 92, [2322]],
  ["neutral-{}-valid", 95, []],
  ["neutral-{}-wrong", 98, [2322]],
  ["neutral-{}-missing", 101, [2345]],
  ["neutral-unknown-valid", 104, []],
  ["neutral-unknown-wrong", 107, [2322]],
  ["neutral-unknown-missing", 110, [2345]],
  ["neutral-undefined-valid", 113, []],
  ["neutral-undefined-wrong", 116, [2322]],
  ["neutral-undefined-missing", 119, [2345]],
  ["neutral-null-valid", 122, []],
  ["neutral-null-wrong", 125, [2322]],
  ["neutral-null-missing", 128, [2345]],
  ["neutral-never-valid", 131, []],
  ["neutral-never-wrong", 134, [2322]],
  ["neutral-never-missing", 137, [2345]],
  ["neutral-any-valid", 140, []],
  ["neutral-any-wrong", 143, [2322]],
  ["neutral-any-missing", 146, [2345]],
  ["neutral-absent-valid", 149, []],
  ["neutral-absent-wrong", 152, [2322]],
  ["neutral-absent-missing", 155, [2345]],
  ["generic-valid", 159, []],
  ["generic-wrong", 160, [2345]],
  ["generated-generic-valid", 164, []],
  ["generated-generic-wrong", 165, [2345]],
  ["internal-generic-valid", 168, []],
  ["internal-generic-wrong", 169, [2322]],
  ["conditional-constructor-valid", 179, []],
  ["conditional-constructor-own-wrong", 180, [2322]],
  ["conditional-constructor-index-wrong", 181, [2322]],
  ["conditional-constructor-index-spread-wrong", 182, [2345]],
  ["conditional-constructor-unknown", 183, []],
  ["conditional-constructor-existing-required-deferred", 184, []],
  ["canonical-required-preserved", 186, [2345]],
  ["untyped-ownership-valid", 189, []],
  ["untyped-ownership-extra-wrong", 190, [2322]],
  ["untyped-ownership-public-wrong", 191, [2322]],
  ["unknown-false-no-listener", 196, []],
  ["unknown-false-listener", 197, []],
  ["unknown-true-no-listener", 198, [2353]],
  ["unknown-true-listener", 199, [2353]],
  ["indexed-wrong-false-no-listener", 200, [2322]],
  ["indexed-wrong-false-listener", 201, [2322, 2322]],
];

function diagnostics(exactOptionalPropertyTypes: boolean) {
  const prelude =
    [
      "type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;",
      "type __EmitProps<E> = { emit: E };",
      "type __VizePublicComponentInputProps = { hidden?: boolean };",
      "type Select<T, K extends PropertyKey> = T extends unknown ? Pick<T, Extract<keyof T, K>> : never;",
      helper.trimEnd(),
    ].join("\n") + "\n";
  const content = prelude + body;
  const offset = prelude.split("\n").length - 1;
  const expected = cases.flatMap(([_name, line, codes]) =>
    codes.map((code) => ({ line: offset + line + 1, code })),
  );
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    types: [],
    skipLibCheck: false,
    target: ts.ScriptTarget.ESNext,
    exactOptionalPropertyTypes,
  };
  const host = ts.createCompilerHost(options);
  const readFile = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : readFile(file));
  const program = ts.createProgram([fixture], options, host);
  const actual = ts.getPreEmitDiagnostics(program).map((d) => {
    assert.equal(d.file?.fileName, fixture, ts.flattenDiagnosticMessageText(d.messageText, "\n"));
    assert.notEqual(d.start, undefined);
    return { line: d.file!.getLineAndCharacterOfPosition(d.start!).line + 1, code: d.code };
  });
  assert.deepEqual(actual, expected, JSON.stringify(cases));
}

test("listener inputs preserve ownership, index values, generics and canonical unknown-prop policy", () => {
  for (const exactOptionalPropertyTypes of [false, true]) diagnostics(exactOptionalPropertyTypes);
});
