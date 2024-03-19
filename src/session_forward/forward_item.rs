use std::sync::Arc;

use bytes::Bytes;
use jt808::JtSubMerger;
use tokio::{sync::{Mutex, RwLock}, net::tcp::OwnedWriteHalf, io::AsyncWriteExt};

use crate::session808::jt808_session::Jt808SessionShared;


pub struct ForwardItem {
    device_session:RwLock<Option<Arc<Jt808SessionShared>>>,
    sender:Arc<Mutex<OwnedWriteHalf>>, 
}

impl ForwardItem {

    pub fn new(sender:Arc<Mutex<OwnedWriteHalf>>) -> Self {
        ForwardItem{
            device_session:RwLock::new(None),
            sender
        }
    }
    
    pub async fn bind_device(&self, device:Arc<Jt808SessionShared>) {
        let mut tt = self.device_session.write().await;
        *tt = Some(device);
    }

    pub async fn forward_recv(forward_item:&Arc<ForwardItem>, mut jtsub:JtSubMerger) {

        let tt = forward_item.device_session.read().await;
        if let Some(device) = tt.clone() {
            device.forward_recv(&mut jtsub, forward_item).await;
        } 
    }

    pub async fn forward_send_bytes(&self, buf:&Bytes) {
        let _ = self.sender.lock().await.write_all(&buf);
    }

    pub async fn forward_send(&self, jtsub:&mut JtSubMerger) {
        match jtsub.end() {
            Some(packages) => {
                for item in packages {
                    let _ = self.sender.lock().await.write_all(&item.get_bytes());
                }
            },
            None => {},
        }
    }

}