use agentproxy::{policy::load_policy, service, service::AppState};
use clap::Parser;
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(name = "agentproxy", version, about = "AgentProxy CLI")]
struct Cli {
    #[arg(long, default_value = "examples/policy.yaml")]
    policy_file: String,
    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,
    #[arg(long, default_value = "examples/puppyrestaurant/puppyrestaurant.db")]
    sqlite_path: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let policy = load_policy(&cli.policy_file).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1)
    });

    let state = AppState::new(policy);
    let app = service::router(state);

    println!(
        "AgentProxy listening on http://{} (sqlite: {})",
        cli.listen, cli.sqlite_path
    );

    let addr: SocketAddr = cli.listen.parse().unwrap_or_else(|error| {
        eprintln!("Invalid listen address: {error}");
        std::process::exit(1)
    });

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

