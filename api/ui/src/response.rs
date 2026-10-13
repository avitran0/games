#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Response {
    pub clicked: bool,
    pub focused: bool,
    pub changed: bool,
}
