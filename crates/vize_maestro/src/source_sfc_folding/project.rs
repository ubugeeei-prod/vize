//! Native SFC comment work over the genuine retained project query.

// This module is exposed only with both experimental source features.
use tower_lsp::lsp_types::FoldingRange;
use vize_l0::Allocator;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{NativeSfcIssue, lower_sfc_native};

use crate::source_project::{ProjectQuery, ProjectQueryResult, SnapshotRefusal};

use super::{SfcFoldingRefusal, sfc_comment_ranges};

/// Original typed producer issues or a failure to project retained comments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfcQueryRefusal {
    Producer(Vec<NativeSfcIssue>),
    Projection(SfcFoldingRefusal),
}

/// Run the actual producer once on this query's once-captured host buffer.
///
/// The arena and borrowed observation live only during this synchronous
/// computation; only owned ranges or the producer's Copy issue records escape.
/// The genuine admission view is the sole input to comment projection. A host
/// snapshot itself grants no File admission.
///
/// The returned project result must be consumed through its genuine `publish`
/// method. That method freshly checks the real URI, global revision, editor
/// version and cancellation under the host read guard. Its callback must publish
/// ready synchronous data without re-entering the project or DocumentStore.
/// No producer owner, document guard or lifecycle lock crosses an await here.
/// Production request routing and full native response history remain separate.
pub async fn query_sfc_comments<'host>(
    query: ProjectQuery<'host>,
    options: DescriptorOptions,
) -> Result<ProjectQueryResult<'host, Result<Vec<FoldingRange>, SfcQueryRefusal>>, SnapshotRefusal>
{
    query
        .run(move |snapshot| async move {
            let arena = Allocator::default();
            let observed = lower_sfc_native(&arena, snapshot.source(), options);
            match observed.admitted() {
                Some(native) => sfc_comment_ranges(&native).map_err(SfcQueryRefusal::Projection),
                None => Err(SfcQueryRefusal::Producer(observed.issues().to_vec())),
            }
        })
        .await
}

#[cfg(test)]
mod tests;
