// Replay this bounded ownership move on current main after a conflict.
// The first commit moves handshake.rs without changing its bytes; the second
// integrates it and extracts the neutral records without changing their fields.
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const contractPath = path.join(root, "crates/vize_extension_contract/src/contract.rs");
const oldHandshake = path.join(root, "crates/vize_extension_contract/src/handshake.rs");
const directory = path.join(root, "crates/vize_carton/src/extension");
const newHandshake = path.join(directory, "handshake.rs");
let contract = readFileSync(contractPath, "utf8");

if (contract.includes("pub use vize_l0::extension::wire::{")) {
  for (const name of ["wire.rs", "handshake.rs", "mod.rs"]) {
    if (!existsSync(path.join(directory, name))) {
      throw new Error(`incomplete ownership move: ${name}`);
    }
  }
  process.stdout.write("extension wire ownership is already integrated\n");
} else {
  const libPath = path.join(root, "crates/vize_carton/src/lib.rs");
  const lib = readFileSync(libPath, "utf8");
  if (lib.split("pub mod expression_guard;\n").length !== 2) {
    throw new Error("foundation module registration changed");
  }
  const handshakeSource = existsSync(newHandshake) ? newHandshake : oldHandshake;
  let handshake = readFileSync(handshakeSource, "utf8");
  for (const anchor of ["use vize_l0::String;", "use crate::contract::{"]) {
    if (handshake.split(anchor).length !== 2) {
      throw new Error(`handshake source boundary changed: ${anchor}`);
    }
  }
  for (const anchor of ["use core::fmt;\n\n", "use vize_l1_to_l2::exemptions;\n"]) {
    if (contract.split(anchor).length !== 2) {
      throw new Error(`contract import boundary changed: ${anchor}`);
    }
  }
  if (existsSync(path.join(directory, "wire.rs")) || existsSync(path.join(directory, "mod.rs"))) {
    throw new Error("foundation extension destination already has code to review");
  }
  const take = (start, end) => {
    if (contract.split(start).length !== 2 || contract.split(end).length !== 2) {
      throw new Error(`source boundary changed: ${start}`);
    }
    const first = contract.indexOf(start);
    const last = contract.indexOf(end, first);
    if (last < first) throw new Error(`reversed source boundary: ${start}`);
    const section = contract.slice(first, last);
    contract = contract.slice(0, first) + contract.slice(last);
    return section;
  };
  const records = take("/// The WIT package this host implements.", "/// `types.diagnostic-part`.");
  const sourceBlock = take(
    "/// `input-lowering.source-block`: one block, whole.",
    "/// `input-lowering.lowered-block`: everything one block lowers to.",
  );
  const errors = take(
    "/// Why a call into a guest returned no value.",
    "/// One input-dialect guest,",
  );
  const spans = take(
    "impl From<vize_l0::Span> for Span {",
    "impl From<&davinci::Diagnostic> for Diagnostic {",
  );
  contract = contract.replace("use core::fmt;\n\n", "");
  contract = contract.replace(
    "use vize_l1_to_l2::exemptions;\n",
    `use vize_l1_to_l2::exemptions;

pub use vize_l0::extension::wire::{
    Capability, GuestError, GuestLimits, L1_PAGE_FEATURE, L1_PAGE_SCHEMA,
    L2_PAGE_FEATURE, L2_PAGE_SCHEMA, LANG_FEATURE_PREFIX, PACKAGE, PROTOCOL_VERSION,
    Page, PartKind, REQUIRED_FEATURES, S1_PAGE_FEATURE, S1_PAGE_SCHEMA,
    S2_PAGE_FEATURE, S2_PAGE_SCHEMA, Severity, SourceBlock, Span, Stage,
};
`,
  );
  mkdirSync(directory, { recursive: true });
  if (!existsSync(newHandshake)) renameSync(oldHandshake, newHandshake);
  handshake = handshake
    .replace("use vize_l0::String;", "use crate::String;")
    .replace("use crate::contract::{", "use super::wire::{");
  writeFileSync(newHandshake, handshake);
  writeFileSync(
    oldHandshake,
    `//! Transitional imports; negotiation is owned by L0.
//! Remove this module with the old contract package after its producer-owned
//! diagnostic conversions and acceptance paths have moved to their levels.

pub use vize_l0::extension::handshake::{
    HandshakeError, Negotiated, negotiate, negotiate_for,
};
`,
  );
  writeFileSync(
    path.join(directory, "wire.rs"),
    `//! Neutral external wire records and protocol constants.
//!
//! These are serialized ABI records, not L1/L2 native IR. Native diagnostics
//! and producer exemption lookup stay at their producing boundary.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::String;

${records}${sourceBlock}${errors}${spans.replaceAll("vize_l0::Span", "crate::Span")}`,
  );
  writeFileSync(
    path.join(directory, "mod.rs"),
    `//! External wire boundaries owned by L0.
//!
//! First-party native producers must use typed level interfaces rather than
//! serializing these pages between levels. Guest runtime packaging remains
//! unfinished until the real L0 extraction can isolate its no_std feature graph.

pub mod handshake;
pub mod wire;
`,
  );
  writeFileSync(contractPath, contract);
  writeFileSync(
    libPath,
    lib.replace("pub mod expression_guard;\n", "pub mod expression_guard;\npub mod extension;\n"),
  );
  process.stdout.write("moved neutral extension wire and negotiation into L0 ownership\n");
}
