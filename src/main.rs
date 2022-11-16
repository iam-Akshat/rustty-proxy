#![deny(clippy::all)]

mod balancer;
mod config;
pub mod util;

use ::futures::future::join_all;
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use balancer::LoadBalancer;
use config::get_config;
use util::targets_status_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args().nth(1).unwrap_or_else(|| {
        "config.json".to_string()
    });

    let config = get_config(config_path).clone();

    let mut handles = Vec::new();

    for proxy_config in config.apps {
        println!("Starting proxy for App: {:?}", proxy_config.name);
        let ports = proxy_config.ports.clone();
        let targets = proxy_config.targets.clone();

        for port in ports {
            let targets = targets.clone();
            handles.push(start_proxy(port, targets));
        }
    }
    join_all(handles).await;

    Ok(())
}

async fn start_proxy(
    server_port: u16,
    targets: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", server_port))
        .await
        .unwrap();

    let the_load_balancer = Arc::new(RwLock::new(LoadBalancer::new(
        &targets,
        &vec![1; targets.len()],
    )));
    // targets_status_check(&mut the_load_balancer).await;
    let mut health_checker_clone = the_load_balancer.clone();
    tokio::spawn(async move {
        loop {
            targets_status_check(&mut health_checker_clone).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    while let Ok((mut inbound, _)) = listener.accept().await {
        // current time in milliseconds
        let thread_lb = the_load_balancer.clone();

        tokio::spawn(async move {
            let target = thread_lb.read().await.get_target();
            let mut outbound = match TcpStream::connect(&target).await {
                Ok(stream) => stream,
                Err(e) => {
                    println!(
                        "Error connecting to target ${:?} {:?}",
                        target,
                        e.to_string()
                    );
                    thread_lb.write().await.update_weight(target, 0);
                    drop(thread_lb);
                    return;
                }
            };

            // this is where we could do some packe manitpulation too
            match tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await {
                Ok((server_bytes, client_bytes)) => {
                    println!(
                        "Connection closed with {} bytes written",
                        server_bytes + client_bytes
                    );
                    println!("Target: {}", target);
                }
                Err(e) => {
                    println!("Error: {:?}", e.to_string());
                }
            }
        });
    }

    Ok(())
}
