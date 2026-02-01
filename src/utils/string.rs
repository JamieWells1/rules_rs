pub trait StringUtils {
    fn at(&self, index: usize) -> Option<char>;
}

impl StringUtils for str {
    fn at(&self, index: usize) -> Option<char> {
        self.chars().nth(index)
    }
}

// Remove first char ('-') and trim
pub fn normalise(string: &str) -> String {
    string
        .trim()
        .chars()
        .skip(1)
        .collect::<String>()
        .trim()
        .to_string()
}
