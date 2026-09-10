#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    pub action: String,
    pub from: String,
    pub to: String,
    pub target_is_end: bool,
}
