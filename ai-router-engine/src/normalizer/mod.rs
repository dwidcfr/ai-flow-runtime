pub trait Normalizer: Send + Sync {
    fn normalize(&self, message: &str) -> String;
}

pub struct DefaultNormalizer;

impl Default for DefaultNormalizer {
    fn default() -> Self {
        Self
    }
}

impl DefaultNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Normalizer for DefaultNormalizer {
    fn normalize(&self, message: &str) -> String {
        message.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_collapses_whitespace() {
        let normalizer = DefaultNormalizer::new();
        assert_eq!(normalizer.normalize("  hello   world  "), "hello world");
    }

    #[test]
    fn preserves_meaning() {
        let normalizer = DefaultNormalizer::new();
        let input = "да, это я";
        assert_eq!(normalizer.normalize(input), "да, это я");
    }
}
