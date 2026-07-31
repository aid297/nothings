pub struct Field {
    pub origin: String,
    pub name: String,
    pub kind: String,
    pub rules: Vec<String>,
    pub is_option: bool,
}

impl Field {
    pub fn new(origin: String, name: String, kind: String, rules: Vec<String>) -> Self {
        Field {
            origin,
            name,
            kind,
            rules,
            is_option: false,
        }
    }
}