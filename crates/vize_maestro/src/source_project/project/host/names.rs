//! Only the actual shared server host authenticates configuration tickets.
#![cfg(feature = "experimental-source-navigation")]
use super::DocumentHost;
use crate::server::NativeNamesSettings;
use crate::source_project::{
    SourceQueryProject,
    navigation::{NavigationRefusal, names::NamesConfiguration, profile::VueConfiguration},
};

pub(super) fn configuration(settings: NativeNamesSettings) -> VueConfiguration {
    VueConfiguration {
        version: settings.version(),
        configured_dialect: settings.configured_dialect(),
        legacy: settings.legacy(),
        patterned: settings.patterned(),
    }
}

impl SourceQueryProject<'_> {
    pub(in crate::source_project) fn with_native_vue_configuration<T>(
        &self,
        read: impl FnOnce(Option<VueConfiguration>) -> T,
    ) -> T {
        match &self.host {
            DocumentHost::Server(state) => {
                state.with_native_names_settings(|settings| read(Some(configuration(settings))))
            }
            _ => read(None),
        }
    }

    pub(in crate::source_project) fn capture_names_parser(
        &self,
    ) -> Result<NamesConfiguration, NavigationRefusal> {
        let DocumentHost::Server(state) = &self.host else {
            return Err(NavigationRefusal::Configuration);
        };
        Ok(NamesConfiguration::Parser(
            state.capture_native_names_parser(),
        ))
    }

    // Publication calls this while the original document read guard is held.
    // The callback only converts captured values or returns an owned response.
    pub(in crate::source_project) fn with_names_configuration<T>(
        &self,
        ticket: &NamesConfiguration,
        publish: impl FnOnce(VueConfiguration) -> T,
    ) -> Result<T, NavigationRefusal> {
        let DocumentHost::Server(state) = &self.host else {
            return Err(NavigationRefusal::Configuration);
        };
        match ticket {
            NamesConfiguration::Parser(ticket) => state
                .with_current_native_names_parser(ticket, |settings| {
                    publish(configuration(settings))
                }),
            NamesConfiguration::Linked(ticket) => state
                .with_current_native_linked_names(ticket, |settings| {
                    publish(configuration(settings))
                }),
        }
        .map_err(NavigationRefusal::from)
    }
}
