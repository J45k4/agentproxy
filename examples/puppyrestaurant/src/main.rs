use agentproxy::{db::SqliteDb, mcp::AgentProxyMcp, policy::load_policy, service};
use axum::Router;
use reqwest::Client;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::json;
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{net::TcpListener, sync::Mutex};
use wgui::{ClientEvent, Item, Wgui, button, text, text_input, vstack};

mod setup;

const INPUT_ID: u32 = 1;
const SEND_ID: u32 = 2;

#[derive(Default)]
struct ChatState {
    draft: String,
    messages: VecDeque<String>,
}

impl ChatState {
    fn push(&mut self, message: String) {
        if self.messages.len() >= 8 {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
}

fn render(state: &ChatState) -> Item {
    let mut body = Vec::new();
    body.push(text("PuppyRestaurant Agent"));
    body.push(text("Raw SQL previews only—no commits yet."));
    body.push(text("Role: customer (limited to ordering + payments)."));
    for message in &state.messages {
        body.push(
            text(message)
                .border("1px solid rgba(255,255,255,0.15)")
                .padding(8)
                .margin_top(4),
        );
    }
    body.push(
        text_input()
            .id(INPUT_ID)
            .svalue(&state.draft)
            .placeholder("Type SQL to preview..."),
    );
    body.push(button("Send").id(SEND_ID));
    vstack(body).into()
}

async fn run_wgui(wgui: Wgui) {
    let mut wgui = wgui;
    let mut clients = HashSet::new();
    let state = Arc::new(Mutex::new(ChatState::default()));
    let client = Client::new();

    while let Some(event) = wgui.next().await {
        match event {
            ClientEvent::Connected { id } => {
                clients.insert(id);
                let view = state.lock().await;
                wgui.render(id, render(&view)).await;
            }
            ClientEvent::Disconnected { id } => {
                clients.remove(&id);
            }
            ClientEvent::OnTextChanged(event) if event.id == INPUT_ID => {
                let mut view = state.lock().await;
                view.draft = event.value;
                for &client_id in &clients {
                    wgui.render(client_id, render(&view)).await;
                }
            }
            ClientEvent::OnClick(event) if event.id == SEND_ID => {
                let mut view = state.lock().await;
                let sql = view.draft.trim().to_string();
                if sql.is_empty() {
                    continue;
                }
                view.draft.clear();
                view.push(format!("You → {sql}"));
                drop(view);
                let view = state.lock().await;
                for &client_id in &clients {
                    wgui.render(client_id, render(&view)).await;
                }
                drop(view);

                let response = client
                    .post("http://127.0.0.1:4000/sql/preview")
                    .json(&json!({
                        "sql": sql,
                        "context": {
                            "actor": "agent:web-ui",
                            "tenant_id": "puppyrestaurant",
                            "role": "customer"
                        }
                    }))
                    .send()
                    .await;
                let message = match response {
                    Ok(resp) => match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if json.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                                format!(
                                    "Agent → preview {} (tables: {:?})",
                                    json["operation"], json["tables"]
                                )
                            } else {
                                format!("Agent → error: {}", json["error"])
                            }
                        }
                        Err(err) => format!("Agent → parse error: {err}"),
                    },
                    Err(err) => format!("Agent → request failed: {err}"),
                };
                let mut view = state.lock().await;
                view.push(message);
                for &client_id in &clients {
                    wgui.render(client_id, render(&view)).await;
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sqlite_path = "examples/puppyrestaurant/puppyrestaurant.db";
    setup::ensure_schema(sqlite_path)?;

    let policy = load_policy("examples/puppyrestaurant/policy.yaml")?;
    let db = SqliteDb::new(sqlite_path)?;
    let state = service::AppState::new(policy).with_db(std::sync::Arc::new(db));
    let wgui = Wgui::new_without_server();
    let wgui_router = wgui.router();
    let mcp_service: StreamableHttpService<AgentProxyMcp, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let state = state.clone();
                move || Ok(AgentProxyMcp::new(state.clone()))
            },
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: Some(Duration::from_secs(15)),
                ..Default::default()
            },
        );
    let router: Router = service::router(state)
        .merge(wgui_router)
        .nest_service("/mcp", mcp_service);

    let addr: SocketAddr = "127.0.0.1:4000".parse()?;
    println!(
        "AgentProxy listening on http://{} (sqlite: {})",
        addr, sqlite_path
    );
    println!("MCP streamable HTTP at http://{}/mcp", addr);

    tokio::spawn(run_wgui(wgui));

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
