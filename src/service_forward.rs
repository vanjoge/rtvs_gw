


use std::sync::{Arc, atomic::Ordering};

use bytes::Bytes;
use tokio::{io::{self, AsyncReadExt}, net::TcpListener, sync::RwLock};

use crate::{session_forward::{forward_parse::{ForwardParse, ReturnType}, forward_session::{ForwardSession, UPDATE}, forward_item::ForwardItem}, session808::jt808_session::Jt808SessionShared};


//2字节body总长度
//控制指令0xffffff开头
//0xffffff00  登录
//0xffffff01  重置sim表
//0xffffff02  添加sim表
//0xffffff03  删除sim表
//[bcdsim 10字节20位]

pub struct ServiceForward {
    pub forward_session:RwLock<Vec<Arc<ForwardSession>>>,
}

impl ServiceForward {
    pub fn new() -> Self {
        ServiceForward {
            forward_session:RwLock::new(Vec::new()),
        }
    }

    pub async fn start(service:Arc<ServiceForward>, addr:&String) -> io::Result<()> {
        
        log::info!("[service-forward]listen addr:{}", addr);
        let listener: TcpListener = TcpListener::bind(addr).await.expect("service device listen failed");
    
       
        let _ = tokio::spawn(async move{
    
            let (socket, _) = listener.accept().await.unwrap();
    
            log::info!("[service-forward]new connect addr:{:?}", socket.peer_addr());
    
            let service = service.clone();
            tokio::spawn(async move{
    
                let (mut reader, writer) = socket.into_split();
    
                //todo:验证流程
                
                let forward_session = Arc::new(ForwardSession::new(writer));
                let mut forward_parse = ForwardParse::new();
                let mut buffer = bytes::BytesMut::with_capacity(8096);
    
                service.forward_session.write().await.push(forward_session.clone());
    
                let mut is_err= false; 
                loop {
                    let n = reader.read_buf(&mut buffer).await.expect("");
                    if n > 0{
                        loop {
                            match forward_parse.parse(&mut buffer) {
                                Ok(ret) => {
                                    match ret {
                                        Some(rt) => {
                                            match rt {
                                                ReturnType::Cmd(t, sims) => {
                                                    forward_session.handle_cmd(t, sims).await;
                                                },
                                                ReturnType::Data(jtsub) => {
                                                    forward_session.handle_data(jtsub).await;
                                                }
                                            }
                                        },
                                        None => {
                                            is_err = true;
                                            break;
                                        },
                                    }
                                },
                                Err(_) => {
                                    break;
                                },
                            }
                        }
    
                    } else {
                        is_err = true;
                    }
    
                    if is_err {
                        break;
                    }
    
                }
    
            });
    
    
        });
    
        Ok(())
    
    }
    
    pub async fn get_forward_sender(service:&Arc<ServiceForward>, sim:&String) -> ForwardSimSender {
        let map_senders = service.forward_session.read().await;
    
        let mut senders:Vec<(Arc<ForwardItem>, i32)> = Vec::new();
        for (_, sender) in map_senders.iter().enumerate() {
            let item = sender.get_item(sim).await;
            match item {
                Some((forward, update)) => {
                    senders.push((forward, update));
                },
                None => {
    
                },
            }
        }
        
        ForwardSimSender{sim: sim.to_string(), service:service.clone(), update_num: UPDATE.load(Ordering::Relaxed), senders }
    }
    

}

pub struct ForwardSimSender {
    sim:String,
    service:Arc<ServiceForward>,
    update_num: i32,
    senders: Vec<(Arc<ForwardItem>, i32)>
}

impl ForwardSimSender {
    
    pub async fn forward_send(&mut self, buf: &Bytes)
    {
        let u = UPDATE.load(Ordering::Relaxed);
        if self.update_num != u {

            let fs = ServiceForward::get_forward_sender(&self.service, &self.sim).await;
            self.senders = fs.senders;
            self.update_num = fs.update_num;
        }

        for (sender, _) in &self.senders {
            sender.forward_send_bytes(buf).await;
        }
    }

    pub async fn bind_device(&mut self, dw_session:Arc<Jt808SessionShared>)
    {
        for (forward, _update) in &self.senders {
            forward.bind_device(dw_session.clone()).await;
        }

    }


}
