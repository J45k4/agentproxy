use axum::{extract::State, Json, response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use wgui::{button, checkbox, table, text, text_input, vstack, Item, Layout, Wgui};

#[derive(Clone)]
pub struct UiState {
    pub messages: Arc<Mutex<VecDeque<String>>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn push(&self, message: String) {
        let mut messages = self.messages.lock().await;
        if messages.len() > 10 {
            messages.pop_front();
        }
        messages.push_back(message);
    }
}

pub fn router_with_ui() -> Router<UiState> {
    let state = UiState::new();
    let ui_clone = state.clone();

    Router::new()
        .route("/", get(move || async move {
            axum::response::Html(include_str!("../agent_ui.html"))
        }))
        .with_state(state)
}
