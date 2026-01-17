use agentproxy::{policy::load_policy, service};
use axum::Router;
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpListener;

mod setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sqlite_path = "examples/puppyrestaurant/puppyrestaurant.db";
    setup::ensure_schema(sqlite_path)?;

    let policy = load_policy("examples/puppyrestaurant/policy.yaml")?;
    let state = service::AppState::new(policy);
    let router: Router = service::router(state);

    let addr: SocketAddr = "127.0.0.1:4000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!(
        "puppyrestaurant example running on http://{} (sqlite: {})",
        addr, sqlite_path
    );

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}









