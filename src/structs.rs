use std::collections::HashMap;

#[derive(Debug)]
pub struct Mirrorlist {
    pub(crate) country: HashMap<String, Vec<String>>,
}
