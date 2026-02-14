/// Convert a title string to a URL-friendly kebab-case slug.
///
/// Examples:
/// - "My Character" -> "my-character"
/// - "The Great Gatsby" -> "the-great-gatsby"
/// - "  Extra   Spaces  " -> "extra-spaces"
/// - "O'Brien & Friends" -> "o-brien-friends"
pub fn slugify(title: &str) -> String {
    slug::slugify(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_title() {
        assert_eq!(slugify("My Character"), "my-character");
    }

    #[test]
    fn multiple_words() {
        assert_eq!(slugify("The Great Gatsby"), "the-great-gatsby");
    }

    #[test]
    fn extra_spaces() {
        assert_eq!(slugify("  Extra   Spaces  "), "extra-spaces");
    }

    #[test]
    fn special_characters() {
        assert_eq!(slugify("O'Brien & Friends"), "o-brien-friends");
    }

    #[test]
    fn unicode() {
        // slug crate transliterates unicode
        let result = slugify("Cafe Resume");
        assert!(!result.is_empty());
        assert!(result.contains("cafe"));
    }

    #[test]
    fn empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn already_slug() {
        assert_eq!(slugify("already-a-slug"), "already-a-slug");
    }
}
