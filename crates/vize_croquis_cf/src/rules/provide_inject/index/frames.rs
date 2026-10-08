use super::AncestorFrame;
use crate::registry::FileId;

pub(super) fn path_from_frame(frames: &[AncestorFrame], mut index: usize) -> Vec<FileId> {
    let mut path = Vec::new();
    while let Some(&frame) = frames.get(index) {
        path.push(frame.current);
        let Some(parent) = frame.parent else {
            break;
        };
        index = parent;
    }
    path
}

pub(super) fn frame_contains(frames: &[AncestorFrame], mut index: usize, needle: FileId) -> bool {
    loop {
        let Some(&frame) = frames.get(index) else {
            return false;
        };
        if frame.current == needle {
            return true;
        }
        let Some(parent) = frame.parent else {
            return false;
        };
        index = parent;
    }
}
