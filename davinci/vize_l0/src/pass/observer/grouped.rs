//! Drive an explicitly requested pipeline with the sequential group cursor.
use super::{FailEvent, PassEvent, PassFailure, PassObserver, PassRemarks, Pipeline};

/// Execute passes and report their established observer and remark events.
/// The next fusion group is discovered after the current one succeeds.
/// Callers with a single walk per group can use [`Pipeline::grouped_passes`]
/// directly and dispatch the borrowed member descriptions over that walk.
pub fn run_grouped_pipeline<O: PassObserver, F>(
    pipeline: &Pipeline,
    observer: &mut O,
    mut step: F,
) -> Result<(), PassFailure>
where
    F: FnMut(&PassEvent<'_>, &mut PassRemarks<'_, '_, O>) -> Result<(), PassFailure>,
{
    observer.before_pipeline(pipeline);
    for (group_index, (group, members)) in pipeline.grouped_passes().enumerate() {
        for offset in 0..members.len() {
            let pass_index = group.start + offset;
            let event = PassEvent {
                pipeline,
                group_index,
                group,
                pass_index,
            };
            observer.before_pass(&event);
            if let Err(failure) = step(&event, &mut PassRemarks::new(observer, event)) {
                observer.on_fail(&FailEvent {
                    pipeline,
                    pass_index: Some(pass_index),
                    reason: failure.reason,
                });
                return Err(failure);
            }
            observer.after_pass(&event);
        }
    }
    observer.after_pipeline(pipeline);
    Ok(())
}
