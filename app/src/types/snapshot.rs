#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorSnapshot {
    pub buf: Vec<char>,
    pub cur: usize,
}
