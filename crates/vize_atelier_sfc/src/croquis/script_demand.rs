//! Result specialization for the same existing script walks.

use vize_croquis::binding_occurrences::BindingOccurrences;
use vize_croquis::{Croquis, Drawer};

pub(super) trait ScriptDemand {
    type Output;

    fn prepare(drawer: Drawer) -> Drawer;
    fn disabled() -> Self::Output;
    fn empty() -> Self::Output;
    fn finish(drawer: Drawer) -> Self::Output;
    fn merge(plain: Self::Output, setup: Self::Output, offset: u32) -> Self::Output;
}

pub(super) struct OrdinaryScript;

impl ScriptDemand for OrdinaryScript {
    type Output = Croquis;

    #[inline]
    fn prepare(drawer: Drawer) -> Drawer {
        drawer
    }

    #[inline]
    fn disabled() -> Croquis {
        Croquis::new()
    }

    #[inline]
    fn empty() -> Croquis {
        Croquis::new()
    }

    #[inline]
    fn finish(drawer: Drawer) -> Croquis {
        drawer.finish()
    }

    #[inline]
    fn merge(plain: Croquis, mut setup: Croquis, offset: u32) -> Croquis {
        setup.shift_script_offsets(offset);
        setup.merge_plain_script(plain);
        setup
    }
}

#[cfg(test)]
mod tests;

pub(super) struct CapturedScript;

impl ScriptDemand for CapturedScript {
    type Output = (Croquis, Option<BindingOccurrences>);

    fn prepare(drawer: Drawer) -> Drawer {
        drawer.with_binding_occurrences()
    }

    fn disabled() -> Self::Output {
        (Croquis::new(), None)
    }

    fn empty() -> Self::Output {
        (Croquis::new(), Some(BindingOccurrences::default()))
    }

    fn finish(drawer: Drawer) -> Self::Output {
        drawer.finish_script_occurrences()
    }

    fn merge(
        (plain, plain_packet): Self::Output,
        (mut summary, setup_packet): Self::Output,
        offset: u32,
    ) -> Self::Output {
        summary.shift_script_offsets(offset);
        summary.merge_plain_script(plain);
        let packet = setup_packet.and_then(|mut setup| {
            setup.shift_script_offsets(offset)?;
            setup.merge(plain_packet?);
            Some(setup)
        });
        (summary, packet)
    }
}
