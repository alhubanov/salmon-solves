#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Clue {
    hint: String
}

impl Clue {
    pub fn new(hint: &String) -> Self {
        Clue { hint: hint.clone() }
    }
}
