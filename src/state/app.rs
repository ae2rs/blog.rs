use crate::content::{Post, PostState, format::highlight::Highlighter};

pub struct AppState {
    post_state: PostState,
}

impl AppState {
    pub fn new() -> Self {
        let highlighter = Highlighter::new();
        let post_state = PostState::new(&highlighter);
        Self { post_state }
    }

    pub fn posts(&self) -> &[&'static Post] {
        self.post_state.posts()
    }

    pub fn post_page(&self, id: &str) -> Option<&String> {
        self.post_state.page(id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
