use std::{collections::HashMap, sync::{Arc, atomic::{Ordering, AtomicI32}}};
use jt808::JtSubMerger;
use tokio::{sync::{RwLock, Mutex}, net::tcp::OwnedWriteHalf};

use super::forward_item::ForwardItem;

//UPDATE 有sim列表更新加1
pub static UPDATE: AtomicI32 = AtomicI32::new(0);

pub struct ForwardSession {
    sender:Arc<Mutex<OwnedWriteHalf>>, 
    map_sims:RwLock<HashMap<String, Arc<ForwardItem>>>,
}

impl ForwardSession {

    pub fn new(sender:OwnedWriteHalf) -> Self {
        ForwardSession{
            sender: Arc::new(Mutex::new(sender)),
            map_sims: RwLock::new(HashMap::new()),
        }
    }

    //处理数据(808)
    pub async fn handle_data(&self, mut jtsub:JtSubMerger) {

        if let Some(jt808) = jtsub.get_first_jt() {
            match self.map_sims.read().await.get(&jt808.sim.to_string()) {
                Some(forward_item) => {
                    ForwardItem::forward_recv(forward_item, jtsub).await;
                },
                None => {
    
                },
            }
        }
    }
    //处理指令
    pub async fn handle_cmd(&self, cmd_type:u8, sims:Vec<String>) {
        if cmd_type == 0x01 {
            self.clear().await;
        }

        if cmd_type == 0x01 || cmd_type == 0x02 {
            self.add(sims).await;
        } else if cmd_type == 0x03 {
            self.sub(sims).await;
        }
    }

    async fn clear(&self) {
        let mut map_sims = self.map_sims.write().await;
        map_sims.clear();

        UPDATE.fetch_add(1, Ordering::Relaxed);
    }

    async fn add(&self, sims:Vec<String>) {
        let mut map_sims = self.map_sims.write().await;
        let mut is_down = false;
        for sim in sims {
            if map_sims.insert(sim, Arc::new(ForwardItem::new(self.sender.clone()))).is_some() {
                is_down = true;
            }
        }

        if is_down {
            UPDATE.fetch_add(1, Ordering::Relaxed);
        }
    }

    async fn sub(&self, sims:Vec<String>) {
        let mut map_sims = self.map_sims.write().await;
        let mut is_down = false;
        for sim in sims {
            if map_sims.remove(&sim).is_some() {
                is_down = true;
            }
        }

        if is_down {
            UPDATE.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn get_item(&self, sim:&String) -> Option<(Arc<ForwardItem>, i32)> {
        let map_sims = self.map_sims.read().await;
        match map_sims.get(sim) {
            Some(item) => {
                return Some((item.clone(), 0));
            },
            None => {
                return None;
            },
        }
    }

}