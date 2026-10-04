use std::path::PathBuf;

use super::{MAX_DUPLICATE_BYTES, MAX_LEAF_BYTES, select};
use crate::batch::{CompositeSourceMap, VirtualFile};
use vize_carton::{FxHashMap, cstr};

#[test]
fn duplicate_bytes_obey_both_fixed_and_relative_caps() {
    for root_bytes in [500_000, 5_000_000] {
        let mut files = Vec::new();
        for name in ["A", "B"] {
            let path = PathBuf::from(cstr!("/project/{name}.vue").as_str());
            files.push(VirtualFile {
                content: "x".repeat(root_bytes).into(),
                source_map: CompositeSourceMap::default(),
                original_path: path.clone(),
                virtual_path: path.with_extension("vue.ts"),
            });
        }
        for index in 0..32 {
            let path = PathBuf::from(cstr!("/project/leaf{index}.ts").as_str());
            let prefix = "export const value = 1;/*";
            files.push(VirtualFile {
                content: cstr!(
                    "{prefix}{}*/",
                    "x".repeat(MAX_LEAF_BYTES - prefix.len() - 2)
                ),
                source_map: CompositeSourceMap::default(),
                original_path: path.clone(),
                virtual_path: path,
            });
        }
        let files: Vec<_> = files.iter().collect();
        let index_by_virtual = files
            .iter()
            .enumerate()
            .map(|(index, file)| (file.virtual_path.as_path(), index))
            .collect::<FxHashMap<_, _>>();
        let specifiers: Vec<_> = (0..32).map(|index| cstr!("./leaf{index}")).collect();
        let references: Vec<_> = specifiers
            .iter()
            .map(|specifier| specifier.as_str())
            .collect();
        let mut imports = vec![references.clone(), references];
        imports.extend((0..32).map(|_| Vec::new()));
        let selected = select(&files, &imports, &index_by_virtual, 8);
        assert!(!selected.is_empty());
        assert!(selected.len() < 32);
        let extra = selected
            .iter()
            .filter_map(|&index| files.get(index))
            .map(|file| file.content.len() * 7)
            .sum::<usize>();
        let relative_budget = files.iter().map(|file| file.content.len()).sum::<usize>() / 100;
        assert!(extra <= MAX_DUPLICATE_BYTES);
        assert!(extra <= relative_budget);
        assert!(
            extra + MAX_LEAF_BYTES * 7 > MAX_DUPLICATE_BYTES.min(relative_budget),
            "the next eligible leaf must exceed the binding budget"
        );
    }
}
