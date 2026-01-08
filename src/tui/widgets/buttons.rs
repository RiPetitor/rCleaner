pub struct Button {
    pub label: String,
    pub shortcut: Option<char>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            shortcut: None,
        }
    }

    pub fn shortcut(mut self, char: char) -> Self {
        self.shortcut = Some(char);
        self
    }
}
