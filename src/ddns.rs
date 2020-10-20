mod tencent;

use anyhow::{format_err, Result};
use async_trait::async_trait;
use parallel_stream::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait UpdateDns: Send + Sync {
    async fn update(self: Arc<Self>, ip: String, domain: String, sub_domain: String) -> Result<()>;
}

enum Domains {
    Tencent {
        config: tencent::Config,
        domains: HashMap<String, Vec<String>>, // map<domain, vec<sub_domain>>
    },
}

impl Domains {
    // TODO 传引用， 优化性能
    fn into_updater_task(self) -> Vec<(Arc<dyn UpdateDns>, String, String)> {
        match self {
            Domains::Tencent { config, domains } => {
                let mut v = Vec::new();
                let updater: Arc<dyn UpdateDns> = Arc::new(tencent::DnsUpdater::new(config));
                for (main, subs) in domains.into_iter() {
                    for sub in subs.into_iter() {
                        v.push((updater.clone(), main.clone(), sub));
                    }
                }
                v
            }
        }
    }
}

struct Config {
    domains: Vec<Domains>,
}

impl Config {
    async fn update_domains(self) -> Result<()> {
        let pub_ip = local_public_ip().await?;
        let update_tasks = self
            .domains
            .into_iter()
            .map(|d| d.into_updater_task())
            .fold(Vec::new(), |mut v, tasks| {
                for task in tasks {
                    v.push((task.0, pub_ip.clone(), task.1, task.2))
                }
                v
            });
        update_tasks
            .into_par_stream()
            .map(|(updater, pub_ip, main, sub)| async move {
                let domain = format!("{}.{}", sub, main);
                let pip = pub_ip.clone();
                match updater.update(pub_ip, main, sub).await {
                    Ok(_) => println!("{} update {} success", domain, pip),
                    Err(e) => println!("{} update {} fail: {}", domain, pip, e),
                }
            })
            .collect::<Vec<()>>()
            .await;

        println!("finish update domains");
        Ok(())
    }
}

async fn local_public_ip() -> Result<String> {
    get_ip_by_cip_cc().await
}

async fn get_ip_by_cip_cc() -> Result<String> {
    let url = "ip.cip.cc";
    surf::get(url)
        .recv_string()
        .await
        .map_err(|err| format_err!("error {}: {}", err.status(), err.to_string()))
}
