use agentproxy::{mcp::AgentProxyMcp, policy::load_policy, service, service::AppState};
use axum::Router;
use clap::Parser;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug, Parser)]
#[command(name = "agentproxy", version, about = "AgentProxy CLI")]
struct Cli {
    #[arg(long, default_value = "examples/policy.yaml")]
    policy_file: String,
    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,
    #[arg(long, default_value = "examples/puppyrestaurant/puppyrestaurant.db")]
    sqlite_path: String,
    #[arg(long)]
    mcp_stdio: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let policy = load_policy(&cli.policy_file).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1)
    });

    let state = AppState::new(policy);

    if cli.mcp_stdio {
        let server = AgentProxyMcp::new(state);
        if let Ok(service) = server.serve(stdio()).await {
            let _ = service.waiting().await;
        }
        return;
    }

    let router: Router = service::router(state);

    println!(
        "AgentProxy listening on http://{} (sqlite: {})",
        cli.listen, cli.sqlite_path
    );

    let addr: SocketAddr = cli.listen.parse().unwrap_or_else(|error| {
        eprintln!("Invalid listen address: {error}");
        std::process::exit(1)
    });

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
