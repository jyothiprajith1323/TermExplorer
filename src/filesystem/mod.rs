//! Filesystem module.

pub mod browser;
pub mod chunked;
pub mod ledger;
pub mod metadata;
pub mod operations;
pub mod search;
pub mod watcher;

pub use browser::{list_dir, DirEntry, EntryKind, SortOrder};
pub use chunked::{load_initial_viewport, stream_dir_chunks, DirChunk};
pub use ledger::{FsOperationLedger, RecordedOperation};
pub use operations::{
    auto_rename_target, copy_entry, create_dir, create_file, delete_dir, delete_entry, delete_file,
    move_entry, rename_entry, restore_from_trash, save_file_atomic, trash_entry,
    trash_entry_tracked, CollisionPolicy,
};
pub use search::{fuzzy_filter, substring_filter};
