use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorkboardPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorkboardSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteEntry {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub position: Option<CorkboardPosition>,
    #[serde(default)]
    pub size: Option<CorkboardSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesConfig {
    pub notes: Vec<NoteEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFrontmatter {
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteContent {
    pub slug: String,
    pub title: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_entry_with_size_roundtrip() {
        let entry = NoteEntry {
            slug: "test".to_string(),
            title: "Test Note".to_string(),
            color: None,
            label: None,
            position: None,
            size: Some(CorkboardSize {
                width: 300.0,
                height: 200.0,
            }),
        };
        let yaml = serde_yaml::to_string(&entry).unwrap();
        let parsed: NoteEntry = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.size.as_ref().unwrap().width, 300.0);
        assert_eq!(parsed.size.as_ref().unwrap().height, 200.0);
    }

    #[test]
    fn test_note_entry_without_size_backward_compat() {
        let yaml = "slug: test\ntitle: Test Note\n";
        let parsed: NoteEntry = serde_yaml::from_str(yaml).unwrap();
        assert!(parsed.size.is_none());
    }

    #[test]
    fn test_corkboard_size_json_camel_case() {
        let size = CorkboardSize {
            width: 250.0,
            height: 180.0,
        };
        let json = serde_json::to_string(&size).unwrap();
        assert!(json.contains("\"width\""));
        assert!(json.contains("\"height\""));
        let parsed: CorkboardSize = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.width, 250.0);
        assert_eq!(parsed.height, 180.0);
    }
}
