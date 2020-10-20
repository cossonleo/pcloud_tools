use super::UpdateDns;

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {

}

pub struct DnsUpdater {
    config: Config,
}

impl DnsUpdater {
    pub fn new(config: Config) -> Self {
        DnsUpdater{config}
    }
}

#[async_trait]
impl UpdateDns for DnsUpdater {
    async fn update(self: Arc<Self>, ip: String, domain: String, sub_domain: String) -> Result<()> {
        Ok(())
    }
}
