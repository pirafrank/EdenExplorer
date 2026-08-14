use crate::core::fs::FileItem;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering::{Greater, Less};

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
    Created,
    Type,
    Deleted,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ListingOrder {
    Windows,
    Linux,
}

impl Default for ListingOrder {
    fn default() -> Self {
        Self::Windows
    }
}

pub fn sort_files(
    files: &mut Vec<FileItem>,
    column: SortColumn,
    ascending: bool,
    listing_order: ListingOrder,
) {
    files.sort_by(|a, b| {
        if listing_order == ListingOrder::Windows && a.is_dir != b.is_dir {
            return if a.is_dir { Less } else { Greater };
        }

        let ord = match column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.file_size.unwrap_or(0).cmp(&b.file_size.unwrap_or(0)),
            SortColumn::Modified => a.modified_time_raw.cmp(&b.modified_time_raw),
            SortColumn::Created => a.created_time_raw.cmp(&b.created_time_raw),
            SortColumn::Deleted => a.deleted_time_raw.cmp(&b.deleted_time_raw),
            SortColumn::Type => match (a.is_dir, b.is_dir) {
                (true, false) => Less,
                (false, true) => Greater,
                (true, true) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                (false, false) => {
                    let a_ext = a
                        .path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let b_ext = b
                        .path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    a_ext.cmp(&b_ext)
                }
            },
        };

        if ascending { ord } else { ord.reverse() }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(name: &str, is_dir: bool, size: u64) -> FileItem {
        FileItem::new(
            name.to_string(),
            PathBuf::from(name),
            is_dir,
            false,
            None,
            Some(size),
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn windows_style_keeps_folders_first_in_both_directions() {
        let mut files = vec![item("a.txt", false, 1), item("z", true, 2)];
        sort_files(&mut files, SortColumn::Name, true, ListingOrder::Windows);
        assert!(files[0].is_dir);
        sort_files(&mut files, SortColumn::Name, false, ListingOrder::Windows);
        assert!(files[0].is_dir);
    }

    #[test]
    fn linux_style_mixes_items_using_selected_column() {
        let mut files = vec![item("folder", true, 9), item("file", false, 1)];
        sort_files(&mut files, SortColumn::Name, true, ListingOrder::Linux);
        assert_eq!(files[0].name, "file");
        sort_files(&mut files, SortColumn::Size, true, ListingOrder::Linux);
        assert_eq!(files[0].name, "file");
        sort_files(&mut files, SortColumn::Size, false, ListingOrder::Linux);
        assert_eq!(files[0].name, "folder");
    }

    #[test]
    fn empty_and_single_inputs_are_unchanged() {
        let mut empty = Vec::new();
        sort_files(&mut empty, SortColumn::Name, true, ListingOrder::Linux);
        let mut single = vec![item("one", false, 1)];
        sort_files(&mut single, SortColumn::Name, false, ListingOrder::Windows);
        assert_eq!(single.len(), 1);
    }
}
