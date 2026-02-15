//! CRDT document model for Sakya projects.
//!
//! Wraps Loro CRDT containers to provide Sakya-specific abstractions
//! for chapters, notes, entities, and project metadata.
//! Handles bidirectional conversion between CRDT state and file-based format.

pub mod chapter;
pub mod entity;
pub mod error;
pub mod export;
pub mod import;
pub mod marks;
pub mod note;
pub mod project;

pub use chapter::{ChapterData, ChapterMetaUpdate, ChapterSummary, SessionData};
pub use entity::EntityData;
pub use error::CrdtError;
pub use export::export_to_files;
pub use import::import_from_files;
pub use marks::{ExpandBehavior, MarkRegistry, MarkSpan, MarkedText};
pub use note::{NoteData, NoteSummary};
pub use project::CrdtProject;

#[cfg(test)]
mod smoke_tests {
    use loro::LoroDoc;

    #[test]
    fn loro_basic_smoke_test() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        map.insert("key", "value").unwrap();
        let val = map.get("key");
        assert!(val.is_some());
        // Check what type we get back
        let val = val.unwrap();
        println!("Value: {:?}", val);
    }

    #[test]
    fn loro_text_smoke_test() {
        let doc = LoroDoc::new();
        let text = doc.get_text("body");
        text.insert(0, "Hello, world!").unwrap();
        let content = text.to_string();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn loro_list_smoke_test() {
        let doc = LoroDoc::new();
        let list = doc.get_list("order");
        list.push("first").unwrap();
        list.push("second").unwrap();
        list.push("third").unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn loro_tree_smoke_test() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("chapters");
        let node_id = tree.create(None).unwrap();
        println!("TreeID: {:?}", node_id);
        let meta = tree.get_meta(node_id).unwrap();
        meta.insert("title", "Chapter 1").unwrap();
        let title = meta.get("title");
        println!("Title from meta: {:?}", title);
        assert!(title.is_some());
    }

    #[test]
    fn loro_snapshot_roundtrip_smoke_test() {
        let doc = LoroDoc::new();
        let map = doc.get_map("meta");
        map.insert("name", "Test Project").unwrap();

        // Export snapshot
        let snapshot = doc.export(loro::ExportMode::Snapshot).unwrap();

        // Import into new doc
        let doc2 = LoroDoc::new();
        doc2.import(&snapshot).unwrap();

        let map2 = doc2.get_map("meta");
        let name = map2.get("name");
        assert!(name.is_some());
    }

    #[test]
    fn loro_updates_sync_smoke_test() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();

        // Get version before changes
        let vv_before = doc_a.oplog_vv();

        // Make changes on doc A
        let map_a = doc_a.get_map("data");
        map_a.insert("from_a", "hello").unwrap();

        // Export updates since the version before changes
        let updates = doc_a
            .export(loro::ExportMode::Updates {
                from: std::borrow::Cow::Borrowed(&vv_before),
            })
            .unwrap();

        // Import into doc B
        doc_b.import(&updates).unwrap();

        let map_b = doc_b.get_map("data");
        let val = map_b.get("from_a");
        assert!(val.is_some());
    }

    #[test]
    fn loro_map_nested_smoke_test() {
        let doc = LoroDoc::new();
        let map = doc.get_map("entities");
        // Check if we can insert a sub-container
        let sub_map = map
            .insert_container("characters", loro::LoroMap::new())
            .unwrap();
        sub_map.insert("name", "Gandalf").unwrap();

        let retrieved = map.get("characters");
        println!("Nested map: {:?}", retrieved);
        assert!(retrieved.is_some());
    }
}
