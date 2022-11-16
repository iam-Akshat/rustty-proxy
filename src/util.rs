use std::sync::{atomic::AtomicBool, Arc};

use futures::future::join_all;

use tokio::{net::TcpStream, sync::RwLock};

use crate::balancer::LoadBalancer;

pub async fn targets_status_check(lb: &mut Arc<RwLock<LoadBalancer>>) {
    //let mut lb = lb.clone();
    let target_weights = lb.read().await.target_weight.clone();
    let mut handles = vec![];
    let is_updated = Arc::new(AtomicBool::new(false));
    for (target, weight) in target_weights.into_iter() {
        let target = target.clone();
        let mut lb = lb.clone();
        let is_updated = is_updated.clone();
        handles.push(tokio::spawn(async move {
            let stream = TcpStream::connect(&target).await;
            match stream {
                Ok(_) => {
                    if weight == 0 {
                        is_updated.store(true, std::sync::atomic::Ordering::Relaxed);
                        println!("{} is UP", target);
                        lb.write().await.update_weight(target, 1);
                    }
                }
                Err(_) => {
                    if weight != 0 {
                        is_updated.store(true, std::sync::atomic::Ordering::Relaxed);
                        println!("{} is DOWN", target);
                        lb.write().await.update_weight(target, 0);
                    }
                }
            }
        }));
    }

    join_all(handles).await;
    if is_updated.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Updated weight");
        lb.read().await.print_targets_state();
    }
}
