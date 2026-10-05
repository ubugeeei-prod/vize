//! Coherent actual settings and retained identities, never syntax admission.
use super::ServerState;
use std::sync::{Arc, atomic::Ordering};
mod settings;
pub(crate) use settings::NativeNamesSettings;

pub(super) struct Generations {
    parser: Arc<()>,
    linked: Arc<()>,
}
impl Default for Generations {
    fn default() -> Self {
        Self {
            parser: Arc::new(()),
            linked: Arc::new(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeNamesConfigurationError {
    ParserChanged,
    RouteChanged,
}

/// Only this actual state can mint a parser ticket; equal settings do not join.
pub(crate) struct NativeNamesParserTicket {
    parser: Arc<()>,
    settings: NativeNamesSettings,
}
/// A real standard-route ticket, distinct from the independent provider API.
pub(crate) struct NativeLinkedNamesTicket {
    parser: NativeNamesParserTicket,
    linked: Arc<()>,
}

/// One coherent actual standard-handler decision before its first await.
pub(crate) enum NativeLinkedNamesRoute {
    Disabled,
    Legacy,
    Native(NativeLinkedNamesTicket),
}

impl ServerState {
    // Caller owns the configuration gate. These reads never reacquire it.
    fn native_names_settings_locked(&self) -> NativeNamesSettings {
        let features = *self.lsp_features.read();
        let version = *self.type_checker_vue_version.read();
        let configured_dialect = *self.dialect_config.read();
        let legacy = *self.type_checker_legacy_vue2.read() || features.legacy_vue2;
        NativeNamesSettings::new(
            version,
            configured_dialect,
            legacy,
            self.experimental_patterned_template.load(Ordering::SeqCst),
            self.native_linked_editing.load(Ordering::SeqCst),
            features.rename,
        )
    }

    /// One coherent value observation for existing profile-only consumers.
    pub(crate) fn native_names_settings(&self) -> NativeNamesSettings {
        self.with_native_names_settings(|settings| settings)
    }

    /// Current-profile extraction may run under an already-held worker map.
    /// Release both guards before retiring controls. This grants no ticket.
    pub(crate) fn with_native_names_settings<T>(
        &self,
        read: impl FnOnce(NativeNamesSettings) -> T,
    ) -> T {
        let _gate = self.native_names.read();
        read(self.native_names_settings_locked())
    }

    pub(crate) fn capture_native_names_parser(&self) -> NativeNamesParserTicket {
        let gate = self.native_names.read();
        NativeNamesParserTicket {
            parser: Arc::clone(&gate.parser),
            settings: self.native_names_settings_locked(),
        }
    }

    pub(crate) fn capture_native_linked_route(&self) -> NativeLinkedNamesRoute {
        let gate = self.native_names.read();
        let settings = self.native_names_settings_locked();
        let (native, rename) = settings.linked_settings();
        if !rename {
            return NativeLinkedNamesRoute::Disabled;
        }
        if !native {
            return NativeLinkedNamesRoute::Legacy;
        }
        NativeLinkedNamesRoute::Native(NativeLinkedNamesTicket {
            parser: NativeNamesParserTicket {
                parser: Arc::clone(&gate.parser),
                settings,
            },
            linked: Arc::clone(&gate.linked),
        })
    }

    /// Used only inside the real current-document publication callback or
    /// map→configuration admission. The callback cannot acquire a worker map,
    /// re-enter the store, retire/wake a worker, parse, or suspend.
    pub(crate) fn with_current_native_names_parser<T>(
        &self,
        ticket: &NativeNamesParserTicket,
        publish: impl FnOnce(NativeNamesSettings) -> T,
    ) -> Result<T, NativeNamesConfigurationError> {
        let gate = self.native_names.read();
        if !Arc::ptr_eq(&gate.parser, &ticket.parser) {
            return Err(NativeNamesConfigurationError::ParserChanged);
        }
        Ok(publish(ticket.settings))
    }

    pub(crate) fn with_current_native_linked_names<T>(
        &self,
        ticket: &NativeLinkedNamesTicket,
        publish: impl FnOnce(NativeNamesSettings) -> T,
    ) -> Result<T, NativeNamesConfigurationError> {
        let gate = self.native_names.read();
        if !Arc::ptr_eq(&gate.parser, &ticket.parser.parser) {
            return Err(NativeNamesConfigurationError::ParserChanged);
        }
        if !Arc::ptr_eq(&gate.linked, &ticket.linked)
            || !self.native_names_settings_locked().linked_enabled()
        {
            return Err(NativeNamesConfigurationError::RouteChanged);
        }
        Ok(publish(ticket.parser.settings))
    }

    /// All relevant real setters enter here; apply uses already-locked helpers.
    /// No I/O, cache invalidation, logging, notification or map/store access.
    pub(super) fn update_native_names_configuration<T>(&self, apply: impl FnOnce() -> T) -> T {
        let mut gate = self.native_names.write();
        let before = self.native_names_settings_locked();
        let result = apply();
        let after = self.native_names_settings_locked();
        if before.parser_settings() != after.parser_settings() {
            gate.parser = Arc::new(());
        }
        if before.linked_settings() != after.linked_settings() {
            gate.linked = Arc::new(());
        }
        result
    }
}
