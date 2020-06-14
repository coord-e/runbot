use crate::model::language::LanguageName;

pub struct Code {
    language: Option<LanguageName>,
    text: String,
}

impl Code {
    pub fn with_language(text: String, language: LanguageName) -> Code {
        Code {
            text,
            language: Some(language),
        }
    }

    pub fn without_language(text: String) -> Code {
        Code {
            text,
            language: None,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn language(&self) -> Option<&LanguageName> {
        self.language.as_ref()
    }
}
