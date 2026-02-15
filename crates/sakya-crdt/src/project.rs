//! CrdtProject: the top-level CRDT document for a Sakya project.

use crate::error::CrdtError;
use loro::{
    Container, LoroDoc, LoroList, LoroMap, LoroText, LoroTree, LoroValue, TreeID, ValueOrContainer,
};
use uuid::Uuid;

/// A Sakya project backed by a Loro CRDT document.
pub struct CrdtProject {
    doc: LoroDoc,
    project_id: Uuid,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract a `String` from a `ValueOrContainer` if it contains a string value.
fn voc_as_str(voc: &ValueOrContainer) -> Option<&str> {
    match voc {
        ValueOrContainer::Value(LoroValue::String(s)) => Some(s.as_ref()),
        _ => None,
    }
}

/// Extract an `i64` from a `ValueOrContainer`.
fn voc_as_i64(voc: &ValueOrContainer) -> Option<i64> {
    match voc {
        ValueOrContainer::Value(LoroValue::I64(n)) => Some(*n),
        _ => None,
    }
}

/// Convert a `ValueOrContainer` to a `serde_json::Value`.
fn voc_to_json(voc: &ValueOrContainer) -> serde_json::Value {
    match voc {
        ValueOrContainer::Value(v) => loro_value_to_json(v),
        ValueOrContainer::Container(c) => match c {
            Container::List(list) => {
                let mut arr = Vec::new();
                for i in 0..list.len() {
                    if let Some(item) = list.get(i) {
                        arr.push(voc_to_json(&item));
                    }
                }
                serde_json::Value::Array(arr)
            }
            Container::Map(map) => {
                let mut obj = serde_json::Map::new();
                for key in map.keys() {
                    if let Some(val) = map.get(&key) {
                        obj.insert(key.to_string(), voc_to_json(&val));
                    }
                }
                serde_json::Value::Object(obj)
            }
            Container::Text(text) => serde_json::Value::String(text.to_string()),
            _ => serde_json::Value::Null,
        },
    }
}

/// Convert a `LoroValue` to a `serde_json::Value`.
fn loro_value_to_json(value: &LoroValue) -> serde_json::Value {
    match value {
        LoroValue::Null => serde_json::Value::Null,
        LoroValue::Bool(b) => serde_json::Value::Bool(*b),
        LoroValue::I64(n) => serde_json::Value::Number((*n).into()),
        LoroValue::Double(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        LoroValue::String(s) => serde_json::Value::String(s.to_string()),
        LoroValue::Binary(_) => serde_json::Value::Null,
        LoroValue::List(arr) => {
            serde_json::Value::Array(arr.iter().map(loro_value_to_json).collect())
        }
        LoroValue::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.to_string(), loro_value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        LoroValue::Container(_) => serde_json::Value::Null,
    }
}

// ── Helpers for finding tree nodes by slug ────────────────────────────────────

/// Find a tree node ID by its slug metadata (skips deleted nodes).
fn find_node_by_slug(tree: &LoroTree, slug: &str) -> Option<TreeID> {
    for node_id in tree.nodes() {
        // Skip deleted nodes
        if tree.is_node_deleted(&node_id).unwrap_or(true) {
            continue;
        }
        if let Ok(meta) = tree.get_meta(node_id) {
            if let Some(node_slug) = meta.get("slug") {
                if voc_as_str(&node_slug) == Some(slug) {
                    return Some(node_id);
                }
            }
        }
    }
    None
}

/// Get the LoroText body from a tree node's metadata.
fn get_body_text(meta: &LoroMap) -> Option<LoroText> {
    match meta.get("body")? {
        ValueOrContainer::Container(Container::Text(text)) => Some(text),
        _ => None,
    }
}

/// Get the LoroMap from a container value.
fn get_sub_map(voc: ValueOrContainer) -> Option<LoroMap> {
    match voc {
        ValueOrContainer::Container(Container::Map(map)) => Some(map),
        _ => None,
    }
}

// ── CrdtProject ──────────────────────────────────────────────────────────────

impl CrdtProject {
    /// Creates a new empty project with initialized CRDT containers.
    pub fn new(project_id: Uuid) -> Self {
        let doc = LoroDoc::new();

        // Initialize container hierarchy
        let _meta = doc.get_map("meta");
        let chapters = doc.get_tree("chapters");
        let notes = doc.get_tree("notes");
        let _entities = doc.get_map("entities");
        let _sessions = doc.get_map("sessions");

        // Enable fractional indexing for ordered trees
        chapters.enable_fractional_index(0);
        notes.enable_fractional_index(0);

        Self { doc, project_id }
    }

    /// Creates a project from an exported snapshot.
    pub fn from_snapshot(project_id: Uuid, bytes: &[u8]) -> Result<Self, CrdtError> {
        let doc = LoroDoc::new();
        doc.import(bytes)?;
        Ok(Self { doc, project_id })
    }

    /// Exports the project as a binary snapshot.
    pub fn export_snapshot(&self) -> Result<Vec<u8>, CrdtError> {
        Ok(self.doc.export(loro::ExportMode::Snapshot)?)
    }

    /// Exports incremental updates since the given version.
    pub fn export_updates(&self, since: &loro::VersionVector) -> Result<Vec<u8>, CrdtError> {
        Ok(self.doc.export(loro::ExportMode::Updates {
            from: std::borrow::Cow::Borrowed(since),
        })?)
    }

    /// Imports updates from another peer.
    pub fn import_updates(&self, bytes: &[u8]) -> Result<(), CrdtError> {
        self.doc.import(bytes)?;
        Ok(())
    }

    /// Gets the current version vector for tracking updates.
    pub fn version_vector(&self) -> loro::VersionVector {
        self.doc.oplog_vv()
    }

    /// Gets the project ID.
    pub fn project_id(&self) -> Uuid {
        self.project_id
    }

    // ── Container accessors ──────────────────────────────────────────────────

    #[allow(dead_code)]
    pub(crate) fn meta_map(&self) -> LoroMap {
        self.doc.get_map("meta")
    }

    pub(crate) fn chapters_tree(&self) -> LoroTree {
        self.doc.get_tree("chapters")
    }

    pub(crate) fn notes_tree(&self) -> LoroTree {
        self.doc.get_tree("notes")
    }

    pub(crate) fn entities_map(&self) -> LoroMap {
        self.doc.get_map("entities")
    }

    #[allow(dead_code)]
    pub(crate) fn sessions_map(&self) -> LoroMap {
        self.doc.get_map("sessions")
    }

    #[allow(dead_code)]
    pub(crate) fn doc(&self) -> &LoroDoc {
        &self.doc
    }

    // ── Chapter CRUD ─────────────────────────────────────────────────────────

    /// Creates a new chapter with the given title. Returns the generated slug.
    pub fn create_chapter(&self, title: &str) -> Result<String, CrdtError> {
        let base_slug = slug::slugify(title);
        let slug = format!(
            "{}-{}",
            base_slug,
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        let chapters = self.chapters_tree();
        let node_id = chapters.create(None)?;
        let meta = chapters.get_meta(node_id)?;

        meta.insert("slug", slug.as_str())?;
        meta.insert("title", title)?;
        meta.insert("status", "draft")?;
        let _body = meta.insert_container("body", LoroText::new())?;

        Ok(slug)
    }

    /// Gets a chapter by its slug.
    pub fn get_chapter(&self, slug: &str) -> Result<crate::ChapterData, CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        let meta = chapters.get_meta(node_id)?;

        let title = meta
            .get("title")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_default();

        let status = meta
            .get("status")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_else(|| "draft".to_string());

        let pov = meta
            .get("pov")
            .and_then(|v| voc_as_str(&v).map(str::to_string));

        let synopsis = meta
            .get("synopsis")
            .and_then(|v| voc_as_str(&v).map(str::to_string));

        let target_words = meta
            .get("targetWords")
            .and_then(|v| voc_as_i64(&v))
            .map(|n| n as u32);

        let body = get_body_text(&meta)
            .map(|t| t.to_string())
            .unwrap_or_default();

        Ok(crate::ChapterData {
            slug: slug.to_string(),
            title,
            status,
            pov,
            synopsis,
            target_words,
            body,
        })
    }

    /// Updates chapter metadata.
    pub fn update_chapter_meta(
        &self,
        slug: &str,
        updates: crate::ChapterMetaUpdate,
    ) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        let meta = chapters.get_meta(node_id)?;

        if let Some(title) = &updates.title {
            meta.insert("title", title.as_str())?;
        }
        if let Some(status) = &updates.status {
            meta.insert("status", status.as_str())?;
        }
        if let Some(pov) = &updates.pov {
            match pov {
                Some(value) => meta.insert("pov", value.as_str())?,
                None => meta.delete("pov")?,
            };
        }
        if let Some(synopsis) = &updates.synopsis {
            match synopsis {
                Some(value) => meta.insert("synopsis", value.as_str())?,
                None => meta.delete("synopsis")?,
            };
        }
        if let Some(target_words) = &updates.target_words {
            match target_words {
                Some(value) => meta.insert("targetWords", *value as i64)?,
                None => meta.delete("targetWords")?,
            };
        }
        Ok(())
    }

    /// Deletes a chapter by its slug.
    pub fn delete_chapter(&self, slug: &str) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        chapters.delete(node_id)?;
        Ok(())
    }

    /// Reorders a chapter to a new position among root children.
    pub fn reorder_chapter(&self, slug: &str, new_index: usize) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();

        let chapter_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;

        // Get root children in order
        let root_children = chapters.children(None::<TreeID>).unwrap_or_default();

        if new_index == 0 {
            chapters.mov_to(chapter_id, None::<TreeID>, 0)?;
        } else if new_index >= root_children.len() {
            // Move to end: use mov_after on the last child
            if let Some(&last) = root_children.last() {
                if last != chapter_id {
                    chapters.mov_after(chapter_id, last)?;
                }
            }
        } else {
            let target = root_children[new_index];
            if target != chapter_id {
                chapters.mov_before(chapter_id, target)?;
            }
        }

        Ok(())
    }

    /// Lists all chapters in order.
    pub fn list_chapters(&self) -> Result<Vec<crate::ChapterSummary>, CrdtError> {
        let chapters = self.chapters_tree();
        let mut summaries = Vec::new();

        let root_children = chapters.children(None::<TreeID>).unwrap_or_default();

        for node_id in root_children {
            let meta = chapters.get_meta(node_id)?;

            let slug = meta
                .get("slug")
                .and_then(|v| voc_as_str(&v).map(str::to_string))
                .unwrap_or_default();

            let title = meta
                .get("title")
                .and_then(|v| voc_as_str(&v).map(str::to_string))
                .unwrap_or_default();

            let status = meta
                .get("status")
                .and_then(|v| voc_as_str(&v).map(str::to_string))
                .unwrap_or_else(|| "draft".to_string());

            summaries.push(crate::ChapterSummary {
                slug,
                title,
                status,
            });
        }

        Ok(summaries)
    }

    /// Inserts text into a chapter's body at the given position.
    pub fn insert_chapter_text(&self, slug: &str, pos: usize, text: &str) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        let meta = chapters.get_meta(node_id)?;
        let body =
            get_body_text(&meta).ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        body.insert(pos, text)?;
        Ok(())
    }

    /// Deletes text from a chapter's body.
    pub fn delete_chapter_text(&self, slug: &str, pos: usize, len: usize) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = find_node_by_slug(&chapters, slug)
            .ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        let meta = chapters.get_meta(node_id)?;
        let body =
            get_body_text(&meta).ok_or_else(|| CrdtError::ChapterNotFound(slug.to_string()))?;
        body.delete(pos, len)?;
        Ok(())
    }

    // ── Note CRUD ────────────────────────────────────────────────────────────

    /// Creates a new note with the given title. Returns the generated slug.
    pub fn create_note(&self, title: &str) -> Result<String, CrdtError> {
        let base_slug = slug::slugify(title);
        let slug = format!(
            "{}-{}",
            base_slug,
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        let notes = self.notes_tree();
        let node_id = notes.create(None)?;
        let meta = notes.get_meta(node_id)?;

        meta.insert("slug", slug.as_str())?;
        meta.insert("title", title)?;
        let _body = meta.insert_container("body", LoroText::new())?;

        Ok(slug)
    }

    /// Gets a note by its slug.
    pub fn get_note(&self, slug: &str) -> Result<crate::NoteData, CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        let meta = notes.get_meta(node_id)?;

        let title = meta
            .get("title")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_default();

        let color = meta
            .get("color")
            .and_then(|v| voc_as_str(&v).map(str::to_string));

        let label = meta
            .get("label")
            .and_then(|v| voc_as_str(&v).map(str::to_string));

        let body = get_body_text(&meta)
            .map(|t| t.to_string())
            .unwrap_or_default();

        Ok(crate::NoteData {
            slug: slug.to_string(),
            title,
            color,
            label,
            body,
        })
    }

    /// Updates note metadata.
    pub fn update_note_meta(
        &self,
        slug: &str,
        title: Option<String>,
        color: Option<Option<String>>,
        label: Option<Option<String>>,
    ) -> Result<(), CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        let meta = notes.get_meta(node_id)?;

        if let Some(title) = &title {
            meta.insert("title", title.as_str())?;
        }
        if let Some(color) = &color {
            match color {
                Some(value) => meta.insert("color", value.as_str())?,
                None => meta.delete("color")?,
            };
        }
        if let Some(label) = &label {
            match label {
                Some(value) => meta.insert("label", value.as_str())?,
                None => meta.delete("label")?,
            };
        }
        Ok(())
    }

    /// Deletes a note by its slug.
    pub fn delete_note(&self, slug: &str) -> Result<(), CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        notes.delete(node_id)?;
        Ok(())
    }

    /// Lists all notes.
    pub fn list_notes(&self) -> Result<Vec<crate::NoteSummary>, CrdtError> {
        let notes = self.notes_tree();
        let mut summaries = Vec::new();

        for node_id in notes.nodes() {
            // Skip deleted nodes
            if notes.is_node_deleted(&node_id).unwrap_or(true) {
                continue;
            }
            let meta = notes.get_meta(node_id)?;

            let slug = meta
                .get("slug")
                .and_then(|v| voc_as_str(&v).map(str::to_string));

            // Only include nodes that have a slug (actual notes, not folders)
            if let Some(slug) = slug {
                let title = meta
                    .get("title")
                    .and_then(|v| voc_as_str(&v).map(str::to_string))
                    .unwrap_or_default();

                let color = meta
                    .get("color")
                    .and_then(|v| voc_as_str(&v).map(str::to_string));

                summaries.push(crate::NoteSummary { slug, title, color });
            }
        }

        Ok(summaries)
    }

    /// Inserts text into a note's body at the given position.
    pub fn insert_note_text(&self, slug: &str, pos: usize, text: &str) -> Result<(), CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        let meta = notes.get_meta(node_id)?;
        let body = get_body_text(&meta).ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        body.insert(pos, text)?;
        Ok(())
    }

    /// Deletes text from a note's body.
    pub fn delete_note_text(&self, slug: &str, pos: usize, len: usize) -> Result<(), CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        let meta = notes.get_meta(node_id)?;
        let body = get_body_text(&meta).ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        body.delete(pos, len)?;
        Ok(())
    }

    // ── Entity CRUD ──────────────────────────────────────────────────────────

    /// Creates a new entity.
    pub fn create_entity(
        &self,
        schema: &str,
        slug: &str,
        title: &str,
        fields: serde_json::Value,
    ) -> Result<(), CrdtError> {
        let entities = self.entities_map();

        // Get or create schema map
        let schema_map = entities.get_or_create_container(schema, LoroMap::new())?;

        // Check if entity already exists
        if schema_map.get(slug).is_some() {
            return Err(CrdtError::Loro(format!(
                "Entity {} already exists in schema {}",
                slug, schema
            )));
        }

        // Create entity map
        let entity_map = schema_map.insert_container(slug, LoroMap::new())?;
        entity_map.insert("title", title)?;

        // Insert fields
        insert_json_fields(&entity_map, &fields)?;

        Ok(())
    }

    /// Gets an entity by schema and slug.
    pub fn get_entity(&self, schema: &str, slug: &str) -> Result<crate::EntityData, CrdtError> {
        let entities = self.entities_map();

        let schema_map = entities
            .get(schema)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::EntityNotFound(format!("{}/{}", schema, slug)))?;

        let entity_map = schema_map
            .get(slug)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::EntityNotFound(format!("{}/{}", schema, slug)))?;

        let title = entity_map
            .get("title")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_default();

        // Extract fields (everything except "title")
        let mut fields = serde_json::Map::new();
        for key in entity_map.keys() {
            if key.as_ref() == "title" {
                continue;
            }
            if let Some(value) = entity_map.get(&key) {
                fields.insert(key.to_string(), voc_to_json(&value));
            }
        }

        Ok(crate::EntityData {
            slug: slug.to_string(),
            title,
            schema_type: schema.to_string(),
            fields: serde_json::Value::Object(fields),
        })
    }

    /// Updates an entity's fields.
    pub fn update_entity(
        &self,
        schema: &str,
        slug: &str,
        field_updates: serde_json::Value,
    ) -> Result<(), CrdtError> {
        let entities = self.entities_map();

        let schema_map = entities
            .get(schema)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::EntityNotFound(format!("{}/{}", schema, slug)))?;

        let entity_map = schema_map
            .get(slug)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::EntityNotFound(format!("{}/{}", schema, slug)))?;

        if let serde_json::Value::Object(obj) = field_updates {
            for (key, value) in obj {
                match value {
                    serde_json::Value::Null => {
                        entity_map.delete(&key)?;
                    }
                    serde_json::Value::String(s) => {
                        entity_map.insert(&key, s.as_str())?;
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            entity_map.insert(&key, i)?;
                        } else if let Some(f) = n.as_f64() {
                            entity_map.insert(&key, f)?;
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        entity_map.insert(&key, b)?;
                    }
                    serde_json::Value::Array(arr) => {
                        entity_map.delete(&key)?;
                        let list = entity_map.insert_container(&key, LoroList::new())?;
                        push_json_array(&list, &arr)?;
                    }
                    serde_json::Value::Object(_) => {} // Skip nested objects for now
                }
            }
        }

        Ok(())
    }

    /// Deletes an entity.
    pub fn delete_entity(&self, schema: &str, slug: &str) -> Result<(), CrdtError> {
        let entities = self.entities_map();

        let schema_map = entities
            .get(schema)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::EntityNotFound(format!("{}/{}", schema, slug)))?;

        if schema_map.get(slug).is_none() {
            return Err(CrdtError::EntityNotFound(format!("{}/{}", schema, slug)));
        }

        schema_map.delete(slug)?;
        Ok(())
    }

    /// Lists all entities for a schema.
    pub fn list_entities(&self, schema: &str) -> Result<Vec<crate::EntityData>, CrdtError> {
        let entities = self.entities_map();
        let mut result = Vec::new();

        if let Some(schema_map) = entities.get(schema).and_then(get_sub_map) {
            for slug in schema_map.keys() {
                if let Ok(entity) = self.get_entity(schema, &slug) {
                    result.push(entity);
                }
            }
        }

        Ok(result)
    }

    /// Lists all schema names that have entities.
    pub fn list_entity_schemas(&self) -> Vec<String> {
        let entities = self.entities_map();
        entities.keys().map(|k| k.to_string()).collect()
    }

    // ── Import methods (explicit slug, all metadata) ─────────────────────

    /// Import a chapter with an explicit slug and all metadata.
    /// Used by `import_from_files` to preserve existing slugs.
    #[allow(clippy::too_many_arguments)]
    pub fn import_chapter(
        &self,
        slug: &str,
        title: &str,
        status: &str,
        pov: Option<&str>,
        synopsis: Option<&str>,
        target_words: Option<u32>,
        body: &str,
    ) -> Result<(), CrdtError> {
        let chapters = self.chapters_tree();
        let node_id = chapters.create(None)?;
        let meta = chapters.get_meta(node_id)?;

        meta.insert("slug", slug)?;
        meta.insert("title", title)?;
        meta.insert("status", status)?;
        if let Some(pov) = pov {
            meta.insert("pov", pov)?;
        }
        if let Some(synopsis) = synopsis {
            meta.insert("synopsis", synopsis)?;
        }
        if let Some(tw) = target_words {
            meta.insert("targetWords", tw as i64)?;
        }
        let body_text = meta.insert_container("body", LoroText::new())?;
        if !body.is_empty() {
            body_text.insert(0, body)?;
        }
        Ok(())
    }

    /// Import a note with an explicit slug and all metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn import_note(
        &self,
        slug: &str,
        title: &str,
        color: Option<&str>,
        label: Option<&str>,
        position_x: Option<f64>,
        position_y: Option<f64>,
        body: &str,
    ) -> Result<(), CrdtError> {
        let notes = self.notes_tree();
        let node_id = notes.create(None)?;
        let meta = notes.get_meta(node_id)?;

        meta.insert("slug", slug)?;
        meta.insert("title", title)?;
        if let Some(color) = color {
            meta.insert("color", color)?;
        }
        if let Some(label) = label {
            meta.insert("label", label)?;
        }
        if let Some(x) = position_x {
            meta.insert("positionX", x)?;
        }
        if let Some(y) = position_y {
            meta.insert("positionY", y)?;
        }
        let body_text = meta.insert_container("body", LoroText::new())?;
        if !body.is_empty() {
            body_text.insert(0, body)?;
        }
        Ok(())
    }

    /// Get full note data including position metadata.
    pub fn get_note_full(
        &self,
        slug: &str,
    ) -> Result<(crate::NoteData, Option<f64>, Option<f64>), CrdtError> {
        let notes = self.notes_tree();
        let node_id = find_node_by_slug(&notes, slug)
            .ok_or_else(|| CrdtError::NoteNotFound(slug.to_string()))?;
        let meta = notes.get_meta(node_id)?;

        let note = self.get_note(slug)?;

        let pos_x = meta.get("positionX").and_then(|v| match v {
            ValueOrContainer::Value(LoroValue::Double(f)) => Some(f),
            _ => None,
        });
        let pos_y = meta.get("positionY").and_then(|v| match v {
            ValueOrContainer::Value(LoroValue::Double(f)) => Some(f),
            _ => None,
        });

        Ok((note, pos_x, pos_y))
    }

    // ── Session CRUD ─────────────────────────────────────────────────────

    /// Import a writing session.
    #[allow(clippy::too_many_arguments)]
    pub fn import_session(
        &self,
        id: &str,
        start: &str,
        end: Option<&str>,
        duration_minutes: Option<f64>,
        words_written: u32,
        chapter_slug: &str,
        sprint_goal: Option<u32>,
    ) -> Result<(), CrdtError> {
        let sessions = self.sessions_map();
        let session_map = sessions.insert_container(id, LoroMap::new())?;
        session_map.insert("start", start)?;
        if let Some(end) = end {
            session_map.insert("end", end)?;
        }
        if let Some(dm) = duration_minutes {
            session_map.insert("durationMinutes", dm)?;
        }
        session_map.insert("wordsWritten", words_written as i64)?;
        session_map.insert("chapterSlug", chapter_slug)?;
        if let Some(goal) = sprint_goal {
            session_map.insert("sprintGoal", goal as i64)?;
        }
        Ok(())
    }

    /// List all session IDs.
    pub fn list_session_ids(&self) -> Vec<String> {
        let sessions = self.sessions_map();
        sessions.keys().map(|k| k.to_string()).collect()
    }

    /// Get session data by ID.
    pub fn get_session(&self, id: &str) -> Result<crate::SessionData, CrdtError> {
        let sessions = self.sessions_map();
        let session_map = sessions
            .get(id)
            .and_then(get_sub_map)
            .ok_or_else(|| CrdtError::Serialization(format!("Session not found: {}", id)))?;

        let start = session_map
            .get("start")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_default();
        let end = session_map
            .get("end")
            .and_then(|v| voc_as_str(&v).map(str::to_string));
        let duration_minutes = session_map.get("durationMinutes").and_then(|v| match v {
            ValueOrContainer::Value(LoroValue::Double(f)) => Some(f),
            _ => None,
        });
        let words_written = session_map
            .get("wordsWritten")
            .and_then(|v| voc_as_i64(&v))
            .unwrap_or(0) as u32;
        let chapter_slug = session_map
            .get("chapterSlug")
            .and_then(|v| voc_as_str(&v).map(str::to_string))
            .unwrap_or_default();
        let sprint_goal = session_map
            .get("sprintGoal")
            .and_then(|v| voc_as_i64(&v))
            .map(|n| n as u32);

        Ok(crate::SessionData {
            id: id.to_string(),
            start,
            end,
            duration_minutes,
            words_written,
            chapter_slug,
            sprint_goal,
        })
    }

    /// Set project metadata.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), CrdtError> {
        self.meta_map().insert(key, value)?;
        Ok(())
    }

    /// Get project metadata.
    pub fn get_meta_value(&self, key: &str) -> Option<String> {
        self.meta_map()
            .get(key)
            .and_then(|v| voc_as_str(&v).map(str::to_string))
    }
}

// ── JSON → Loro field insertion ──────────────────────────────────────────────

fn insert_json_fields(map: &LoroMap, fields: &serde_json::Value) -> Result<(), CrdtError> {
    if let serde_json::Value::Object(obj) = fields {
        for (key, value) in obj {
            match value {
                serde_json::Value::String(s) => {
                    map.insert(key, s.as_str())?;
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        map.insert(key, i)?;
                    } else if let Some(f) = n.as_f64() {
                        map.insert(key, f)?;
                    }
                }
                serde_json::Value::Bool(b) => {
                    map.insert(key, *b)?;
                }
                serde_json::Value::Array(arr) => {
                    let list = map.insert_container(key, LoroList::new())?;
                    push_json_array(&list, arr)?;
                }
                _ => {} // Skip null and nested objects
            }
        }
    }
    Ok(())
}

fn push_json_array(list: &LoroList, arr: &[serde_json::Value]) -> Result<(), CrdtError> {
    for item in arr {
        match item {
            serde_json::Value::String(s) => list.push(s.as_str())?,
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    list.push(i)?;
                } else if let Some(f) = n.as_f64() {
                    list.push(f)?;
                }
            }
            serde_json::Value::Bool(b) => list.push(*b)?,
            _ => {} // Skip complex nested values
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_initializes_containers() {
        let project_id = Uuid::new_v4();
        let project = CrdtProject::new(project_id);

        assert_eq!(project.project_id(), project_id);

        // These should not panic
        let _meta = project.meta_map();
        let _chapters = project.chapters_tree();
        let _notes = project.notes_tree();
        let _entities = project.entities_map();
        let _sessions = project.sessions_map();
    }

    #[test]
    fn snapshot_round_trip() {
        let project_id = Uuid::new_v4();
        let project1 = CrdtProject::new(project_id);

        let meta = project1.meta_map();
        meta.insert("title", "Test Project").unwrap();
        meta.insert("author", "Test Author").unwrap();

        let snapshot = project1.export_snapshot().unwrap();
        let project2 = CrdtProject::from_snapshot(project_id, &snapshot).unwrap();

        let meta2 = project2.meta_map();
        assert_eq!(
            meta2
                .get("title")
                .and_then(|v| voc_as_str(&v).map(str::to_string)),
            Some("Test Project".to_string())
        );
        assert_eq!(
            meta2
                .get("author")
                .and_then(|v| voc_as_str(&v).map(str::to_string)),
            Some("Test Author".to_string())
        );
    }

    #[test]
    fn incremental_updates() {
        let project_id = Uuid::new_v4();
        let project1 = CrdtProject::new(project_id);
        let project2 = CrdtProject::new(project_id);

        let version_before = project1.version_vector();

        project1
            .meta_map()
            .insert("title", "Synced Project")
            .unwrap();

        let updates = project1.export_updates(&version_before).unwrap();
        project2.import_updates(&updates).unwrap();

        let meta2 = project2.meta_map();
        assert_eq!(
            meta2
                .get("title")
                .and_then(|v| voc_as_str(&v).map(str::to_string)),
            Some("Synced Project".to_string())
        );
    }

    #[test]
    fn container_hierarchy() {
        let project = CrdtProject::new(Uuid::new_v4());

        let entities = project.entities_map();
        let characters = entities
            .insert_container("characters", LoroMap::new())
            .unwrap();
        characters.insert("protagonist", "Alice").unwrap();

        if let Some(ValueOrContainer::Container(Container::Map(chars))) = entities.get("characters")
        {
            assert_eq!(
                chars
                    .get("protagonist")
                    .and_then(|v| voc_as_str(&v).map(str::to_string)),
                Some("Alice".to_string())
            );
        } else {
            panic!("Expected container value");
        }
    }

    #[test]
    fn chapter_tree_node_creation() {
        let project = CrdtProject::new(Uuid::new_v4());
        let chapters = project.chapters_tree();

        let node_id = chapters.create(None).unwrap();
        let meta = chapters.get_meta(node_id).unwrap();

        meta.insert("title", "Chapter 1").unwrap();
        meta.insert("slug", "chapter-1").unwrap();
        meta.insert("status", "draft").unwrap();

        let body = meta.insert_container("body", LoroText::new()).unwrap();
        body.insert(0, "Once upon a time...").unwrap();

        assert_eq!(
            meta.get("title")
                .and_then(|v| voc_as_str(&v).map(str::to_string)),
            Some("Chapter 1".to_string())
        );

        if let Some(ValueOrContainer::Container(Container::Text(text))) = meta.get("body") {
            assert_eq!(text.to_string(), "Once upon a time...");
        } else {
            panic!("Expected body container");
        }
    }

    #[test]
    fn note_tree_structure() {
        let project = CrdtProject::new(Uuid::new_v4());
        let notes = project.notes_tree();

        let node_id = notes.create(None).unwrap();
        let meta = notes.get_meta(node_id).unwrap();

        meta.insert("title", "Character Notes").unwrap();
        meta.insert("slug", "character-notes").unwrap();
        meta.insert("color", "#ff0000").unwrap();

        let body = meta.insert_container("body", LoroText::new()).unwrap();
        body.insert(0, "Alice is brave and curious.").unwrap();

        assert_eq!(
            meta.get("color")
                .and_then(|v| voc_as_str(&v).map(str::to_string)),
            Some("#ff0000".to_string())
        );
    }

    #[test]
    fn sessions_map() {
        let project = CrdtProject::new(Uuid::new_v4());
        let sessions = project.sessions_map();

        let session_id = Uuid::new_v4().to_string();
        let session_data = sessions
            .insert_container(&session_id, LoroMap::new())
            .unwrap();
        session_data.insert("start_time", 1234567890i64).unwrap();
        session_data.insert("word_count", 500i64).unwrap();

        if let Some(ValueOrContainer::Container(Container::Map(data))) = sessions.get(&session_id) {
            assert_eq!(
                data.get("word_count").and_then(|v| voc_as_i64(&v)),
                Some(500)
            );
        } else {
            panic!("Expected session data");
        }
    }

    #[test]
    fn version_tracking() {
        let project = CrdtProject::new(Uuid::new_v4());

        let v1 = project.version_vector();
        project.meta_map().insert("version", 1i64).unwrap();
        let v2 = project.version_vector();

        assert_ne!(v1, v2);
    }

    // ── Chapter CRUD tests ───────────────────────────────────────────────────

    #[test]
    fn create_chapter() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug = project.create_chapter("Chapter One").unwrap();
        assert!(slug.starts_with("chapter-one-"));

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.title, "Chapter One");
        assert_eq!(chapter.status, "draft");
        assert_eq!(chapter.body, "");
        assert_eq!(chapter.pov, None);
        assert_eq!(chapter.synopsis, None);
        assert_eq!(chapter.target_words, None);
    }

    #[test]
    fn get_chapter_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());
        let result = project.get_chapter("non-existent");
        assert!(matches!(result, Err(CrdtError::ChapterNotFound(_))));
    }

    #[test]
    fn update_chapter_meta() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Chapter Two").unwrap();

        let updates = crate::ChapterMetaUpdate {
            title: Some("Chapter Two: The Journey".to_string()),
            status: Some("revised".to_string()),
            pov: Some(Some("Alice".to_string())),
            synopsis: Some(Some("Alice begins her journey.".to_string())),
            target_words: Some(Some(2500)),
        };

        project.update_chapter_meta(&slug, updates).unwrap();

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.title, "Chapter Two: The Journey");
        assert_eq!(chapter.status, "revised");
        assert_eq!(chapter.pov, Some("Alice".to_string()));
        assert_eq!(
            chapter.synopsis,
            Some("Alice begins her journey.".to_string())
        );
        assert_eq!(chapter.target_words, Some(2500));
    }

    #[test]
    fn update_chapter_meta_clear_fields() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Chapter Three").unwrap();

        let updates1 = crate::ChapterMetaUpdate {
            pov: Some(Some("Bob".to_string())),
            synopsis: Some(Some("A summary.".to_string())),
            target_words: Some(Some(1000)),
            ..Default::default()
        };
        project.update_chapter_meta(&slug, updates1).unwrap();

        let updates2 = crate::ChapterMetaUpdate {
            pov: Some(None),
            synopsis: Some(None),
            target_words: Some(None),
            ..Default::default()
        };
        project.update_chapter_meta(&slug, updates2).unwrap();

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.pov, None);
        assert_eq!(chapter.synopsis, None);
        assert_eq!(chapter.target_words, None);
    }

    #[test]
    fn delete_chapter() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Chapter to Delete").unwrap();

        project.delete_chapter(&slug).unwrap();

        let result = project.get_chapter(&slug);
        assert!(matches!(result, Err(CrdtError::ChapterNotFound(_))));

        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters.len(), 0);
    }

    #[test]
    fn reorder_chapters() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug1 = project.create_chapter("Chapter 1").unwrap();
        let slug2 = project.create_chapter("Chapter 2").unwrap();
        let slug3 = project.create_chapter("Chapter 3").unwrap();

        // Initial order: 1, 2, 3
        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters[0].slug, slug1);
        assert_eq!(chapters[1].slug, slug2);
        assert_eq!(chapters[2].slug, slug3);

        // Move chapter 3 to beginning → 3, 1, 2
        project.reorder_chapter(&slug3, 0).unwrap();
        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters[0].slug, slug3);
        assert_eq!(chapters[1].slug, slug1);
        assert_eq!(chapters[2].slug, slug2);

        // Move chapter 1 to end → 3, 2, 1
        project.reorder_chapter(&slug1, 3).unwrap();
        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters[0].slug, slug3);
        assert_eq!(chapters[1].slug, slug2);
        assert_eq!(chapters[2].slug, slug1);
    }

    #[test]
    fn list_chapters() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug1 = project.create_chapter("First Chapter").unwrap();
        let slug2 = project.create_chapter("Second Chapter").unwrap();

        project
            .update_chapter_meta(
                &slug2,
                crate::ChapterMetaUpdate {
                    status: Some("final".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].slug, slug1);
        assert_eq!(chapters[0].title, "First Chapter");
        assert_eq!(chapters[0].status, "draft");
        assert_eq!(chapters[1].slug, slug2);
        assert_eq!(chapters[1].title, "Second Chapter");
        assert_eq!(chapters[1].status, "final");
    }

    #[test]
    fn insert_chapter_text() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Text Chapter").unwrap();

        project
            .insert_chapter_text(&slug, 0, "Once upon a time")
            .unwrap();
        project
            .insert_chapter_text(&slug, 16, ", there was a story.")
            .unwrap();

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.body, "Once upon a time, there was a story.");
    }

    #[test]
    fn delete_chapter_text() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Edit Chapter").unwrap();

        project
            .insert_chapter_text(&slug, 0, "The quick brown fox jumps over the lazy dog.")
            .unwrap();

        // Delete "brown "
        project.delete_chapter_text(&slug, 10, 6).unwrap();

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.body, "The quick fox jumps over the lazy dog.");
    }

    #[test]
    fn chapter_text_operations_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());

        let result1 = project.insert_chapter_text("non-existent", 0, "text");
        assert!(matches!(result1, Err(CrdtError::ChapterNotFound(_))));

        let result2 = project.delete_chapter_text("non-existent", 0, 5);
        assert!(matches!(result2, Err(CrdtError::ChapterNotFound(_))));
    }

    #[test]
    fn chapter_with_special_characters() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug = project
            .create_chapter("Chapter & Special: \"Characters\"!")
            .unwrap();
        assert!(slug.starts_with("chapter-special-characters-"));

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.title, "Chapter & Special: \"Characters\"!");
    }

    #[test]
    fn concurrent_chapter_updates() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_chapter("Concurrent Chapter").unwrap();

        project.insert_chapter_text(&slug, 0, "Start ").unwrap();
        project.insert_chapter_text(&slug, 6, "middle ").unwrap();
        project.insert_chapter_text(&slug, 13, "end").unwrap();

        let chapter = project.get_chapter(&slug).unwrap();
        assert_eq!(chapter.body, "Start middle end");
    }

    // ── Note CRUD tests ──────────────────────────────────────────────────────

    #[test]
    fn create_note() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug = project.create_note("Character Notes").unwrap();
        assert!(slug.starts_with("character-notes-"));

        let note = project.get_note(&slug).unwrap();
        assert_eq!(note.title, "Character Notes");
        assert_eq!(note.body, "");
        assert_eq!(note.color, None);
        assert_eq!(note.label, None);
    }

    #[test]
    fn get_note_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());
        let result = project.get_note("non-existent");
        assert!(matches!(result, Err(CrdtError::NoteNotFound(_))));
    }

    #[test]
    fn update_note_meta() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_note("Plot Ideas").unwrap();

        project
            .update_note_meta(
                &slug,
                Some("Plot Ideas - Revised".to_string()),
                Some(Some("#ff0000".to_string())),
                Some(Some("important".to_string())),
            )
            .unwrap();

        let note = project.get_note(&slug).unwrap();
        assert_eq!(note.title, "Plot Ideas - Revised");
        assert_eq!(note.color, Some("#ff0000".to_string()));
        assert_eq!(note.label, Some("important".to_string()));
    }

    #[test]
    fn update_note_meta_clear_fields() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_note("Temporary Note").unwrap();

        project
            .update_note_meta(
                &slug,
                None,
                Some(Some("#00ff00".to_string())),
                Some(Some("draft".to_string())),
            )
            .unwrap();

        project
            .update_note_meta(&slug, None, Some(None), Some(None))
            .unwrap();

        let note = project.get_note(&slug).unwrap();
        assert_eq!(note.color, None);
        assert_eq!(note.label, None);
    }

    #[test]
    fn delete_note() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_note("Note to Delete").unwrap();

        project.delete_note(&slug).unwrap();

        let result = project.get_note(&slug);
        assert!(matches!(result, Err(CrdtError::NoteNotFound(_))));
    }

    #[test]
    fn list_notes() {
        let project = CrdtProject::new(Uuid::new_v4());

        let slug1 = project.create_note("First Note").unwrap();
        let slug2 = project.create_note("Second Note").unwrap();

        project
            .update_note_meta(&slug2, None, Some(Some("#0000ff".to_string())), None)
            .unwrap();

        let notes = project.list_notes().unwrap();
        assert_eq!(notes.len(), 2);

        let note1 = notes.iter().find(|n| n.slug == slug1).unwrap();
        assert_eq!(note1.title, "First Note");
        assert_eq!(note1.color, None);

        let note2 = notes.iter().find(|n| n.slug == slug2).unwrap();
        assert_eq!(note2.title, "Second Note");
        assert_eq!(note2.color, Some("#0000ff".to_string()));
    }

    #[test]
    fn insert_note_text() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_note("Text Note").unwrap();

        project.insert_note_text(&slug, 0, "Important: ").unwrap();
        project
            .insert_note_text(&slug, 11, "Remember this")
            .unwrap();

        let note = project.get_note(&slug).unwrap();
        assert_eq!(note.body, "Important: Remember this");
    }

    #[test]
    fn delete_note_text() {
        let project = CrdtProject::new(Uuid::new_v4());
        let slug = project.create_note("Edit Note").unwrap();

        project
            .insert_note_text(&slug, 0, "This is a very important note.")
            .unwrap();

        // Delete "very "
        project.delete_note_text(&slug, 10, 5).unwrap();

        let note = project.get_note(&slug).unwrap();
        assert_eq!(note.body, "This is a important note.");
    }

    #[test]
    fn note_text_operations_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());

        let result1 = project.insert_note_text("non-existent", 0, "text");
        assert!(matches!(result1, Err(CrdtError::NoteNotFound(_))));

        let result2 = project.delete_note_text("non-existent", 0, 5);
        assert!(matches!(result2, Err(CrdtError::NoteNotFound(_))));
    }

    // ── Entity CRUD tests ────────────────────────────────────────────────────

    #[test]
    fn create_entity() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity(
                "characters",
                "alice",
                "Alice",
                serde_json::json!({
                    "age": 28,
                    "role": "protagonist",
                    "traits": ["brave", "curious"]
                }),
            )
            .unwrap();

        let entity = project.get_entity("characters", "alice").unwrap();
        assert_eq!(entity.title, "Alice");
        assert_eq!(entity.schema_type, "characters");
        assert_eq!(entity.fields["role"], "protagonist");
        assert_eq!(entity.fields["age"], 28);
    }

    #[test]
    fn entity_duplicate_rejected() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity("characters", "alice", "Alice", serde_json::json!({}))
            .unwrap();

        let result = project.create_entity("characters", "alice", "Alice 2", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn get_entity_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());
        let result = project.get_entity("characters", "nobody");
        assert!(matches!(result, Err(CrdtError::EntityNotFound(_))));
    }

    #[test]
    fn update_entity() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity(
                "characters",
                "bob",
                "Bob",
                serde_json::json!({ "age": 30, "role": "mentor" }),
            )
            .unwrap();

        project
            .update_entity(
                "characters",
                "bob",
                serde_json::json!({ "age": 31, "role": serde_json::Value::Null }),
            )
            .unwrap();

        let entity = project.get_entity("characters", "bob").unwrap();
        assert_eq!(entity.fields["age"], 31);
        assert!(entity.fields.get("role").is_none());
    }

    #[test]
    fn delete_entity() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity("locations", "castle", "The Castle", serde_json::json!({}))
            .unwrap();

        project.delete_entity("locations", "castle").unwrap();

        let result = project.get_entity("locations", "castle");
        assert!(matches!(result, Err(CrdtError::EntityNotFound(_))));
    }

    #[test]
    fn delete_entity_not_found() {
        let project = CrdtProject::new(Uuid::new_v4());

        let result = project.delete_entity("characters", "nobody");
        assert!(matches!(result, Err(CrdtError::EntityNotFound(_))));
    }

    #[test]
    fn list_entities() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity("characters", "alice", "Alice", serde_json::json!({}))
            .unwrap();
        project
            .create_entity("characters", "bob", "Bob", serde_json::json!({}))
            .unwrap();
        project
            .create_entity("locations", "castle", "Castle", serde_json::json!({}))
            .unwrap();

        let characters = project.list_entities("characters").unwrap();
        assert_eq!(characters.len(), 2);

        let locations = project.list_entities("locations").unwrap();
        assert_eq!(locations.len(), 1);

        let empty = project.list_entities("items").unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn entity_with_array_field() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity(
                "characters",
                "alice",
                "Alice",
                serde_json::json!({ "traits": ["brave", "kind", "clever"] }),
            )
            .unwrap();

        let entity = project.get_entity("characters", "alice").unwrap();
        let traits = entity.fields["traits"].as_array().unwrap();
        assert_eq!(traits.len(), 3);
        assert_eq!(traits[0], "brave");
        assert_eq!(traits[1], "kind");
        assert_eq!(traits[2], "clever");
    }

    #[test]
    fn entity_update_array_field() {
        let project = CrdtProject::new(Uuid::new_v4());

        project
            .create_entity(
                "characters",
                "alice",
                "Alice",
                serde_json::json!({ "traits": ["brave"] }),
            )
            .unwrap();

        project
            .update_entity(
                "characters",
                "alice",
                serde_json::json!({ "traits": ["brave", "kind"] }),
            )
            .unwrap();

        let entity = project.get_entity("characters", "alice").unwrap();
        let traits = entity.fields["traits"].as_array().unwrap();
        assert_eq!(traits.len(), 2);
    }

    // ── Cross-domain tests ───────────────────────────────────────────────────

    #[test]
    fn two_doc_sync_chapters() {
        let pid = Uuid::new_v4();
        let p1 = CrdtProject::new(pid);
        let p2 = CrdtProject::new(pid);

        let vv = p1.version_vector();
        let slug = p1.create_chapter("Remote Chapter").unwrap();
        p1.insert_chapter_text(&slug, 0, "Hello from device 1")
            .unwrap();

        let updates = p1.export_updates(&vv).unwrap();
        p2.import_updates(&updates).unwrap();

        let chapter = p2.get_chapter(&slug).unwrap();
        assert_eq!(chapter.title, "Remote Chapter");
        assert_eq!(chapter.body, "Hello from device 1");
    }

    #[test]
    fn two_doc_sync_entities() {
        let pid = Uuid::new_v4();
        let p1 = CrdtProject::new(pid);
        let p2 = CrdtProject::new(pid);

        let vv = p1.version_vector();
        p1.create_entity(
            "characters",
            "alice",
            "Alice",
            serde_json::json!({ "age": 28 }),
        )
        .unwrap();

        let updates = p1.export_updates(&vv).unwrap();
        p2.import_updates(&updates).unwrap();

        let entity = p2.get_entity("characters", "alice").unwrap();
        assert_eq!(entity.title, "Alice");
        assert_eq!(entity.fields["age"], 28);
    }

    #[test]
    fn empty_project_operations() {
        let project = CrdtProject::new(Uuid::new_v4());

        assert_eq!(project.list_chapters().unwrap().len(), 0);
        assert_eq!(project.list_notes().unwrap().len(), 0);
        assert_eq!(project.list_entities("any").unwrap().len(), 0);
    }
}
