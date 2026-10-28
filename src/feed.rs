use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use crate::protocol::{C2Message, TacticalContact};

pub struct TacticalFeed {
    pub contacts: Arc<Mutex<Vec<TacticalContact>>>,
    _runtime: Runtime,
}

impl TacticalFeed {
    pub fn connect(url: &str) -> Self {
        let contacts: Arc<Mutex<Vec<TacticalContact>>> = Arc::new(Mutex::new(Vec::new()));
        let contacts_clone = contacts.clone();
        let url = url.to_string();

        let rt = Runtime::new().expect("tokio runtime");
        rt.spawn(async move {
            loop {
                match tokio_tungstenite::connect_async(&url).await {
                    Ok((ws, _)) => {
                        log::info!("C2 feed connected: {}", url);
                        let (_, mut read) = ws.split();
                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(m) if m.is_text() => {
                                    if let Ok(c2) = serde_json::from_str::<C2Message>(m.to_text().unwrap_or("")) {
                                        match c2 {
                                            C2Message::ContactUpdate(list) => {
                                                if let Ok(mut lock) = contacts_clone.lock() {
                                                    *lock = list;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::warn!("C2 feed error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("C2 feed connect failed (will retry): {}", e);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });

        Self { contacts, _runtime: rt }
    }

    pub fn snapshot(&self) -> Vec<TacticalContact> {
        self.contacts.lock().map(|c| c.clone()).unwrap_or_default()
    }
}
