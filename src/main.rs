mod balancer;
mod config;
pub mod util;

use ::futures::future::{join_all, try_join};
use async_recursion::async_recursion;
use log::{debug, info};
use std::sync::Arc;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use balancer::LoadBalancer;
use config::get_config;
use util::targets_status_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.json".to_string());

    let config = get_config(config_path).clone();

    let mut handles = Vec::new();

    for proxy_config in config.apps {
        println!("Starting proxy for App: {:?}", proxy_config.name);
        let ports = proxy_config.ports.clone();
        let targets = proxy_config.targets.clone();
        
        let load_balancer = Arc::new(RwLock::new(LoadBalancer::new(
            &targets,
            &vec![1; targets.len()],
        )));

        let mut health_checker_clone = load_balancer.clone();
        tokio::spawn(async move {
            loop {
                targets_status_check(&mut health_checker_clone).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
        for port in ports {
            handles.push(start_proxy(port, load_balancer.clone()));
        }
    }
    join_all(handles).await;

    Ok(())
}

async fn start_proxy(
    server_port: u16,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", server_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error binding to port: {}", server_port);
            return Err(Box::new(e));
        }
    };

    while let Ok((inbound, _)) = listener.accept().await {
        let thread_lb = load_balancer.clone();

        if thread_lb.read().await.is_active == false {
            println!("No active targets");
            continue;
        }

        tokio::spawn(async move {
            match handle_connection(inbound, thread_lb, 0).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error handling connection: {}", e);
                }
            };
        });
    }

    Ok(())
}

#[async_recursion]
async fn handle_connection(
    mut inbound: TcpStream,
    load_balancer: Arc<RwLock<LoadBalancer>>,
    try_count: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    if try_count > 3 {
        info!("Enough retries, giving up");
        return Ok(());
    }
    let target = load_balancer.read().await.get_target();
    let mut outbound = match TcpStream::connect(&target).await {
        Ok(stream) => stream,
        Err(e) => {
            println!(
                "Error connecting to target ${:?} {:?}",
                target,
                e.to_string()
            );
            load_balancer.write().await.update_weight(target, 0);
            return handle_connection(inbound, load_balancer, try_count + 1).await;
        }
    };

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();
    // packet manipulation possible here
    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    let (bytes_c2s, bytes_s2c) = try_join(client_to_server, server_to_client).await?;
    debug!(
        "Connection closed with {} bytes received and {} bytes sent",
        bytes_c2s, bytes_s2c
    );

    Ok(())
}
