

use axum::async_trait;
use jt808::JtPackage;
use tokio::{io::{self, AsyncReadExt}, net::TcpListener, sync::Mutex, time::timeout};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{session808::{jt808_session::{Jt808SessionShared, Jt808Session}, jt808_parse::Jt808DeserializeAndPackUp}, service_forward::ServiceForward};

static GLOBAL_DATA: std::sync::Mutex<Option<HashMap<String, Arc<Jt808SessionShared>>>> = std::sync::Mutex::new(None);

pub trait GetSender {
    fn get_sender(&self, sim:&String) -> Option<Arc<Jt808SessionShared>>;
}
pub struct ServiceJt808 {
    m_map_sessions: Mutex<HashMap<String, Arc<Jt808SessionShared>>>,
}

impl ServiceJt808 {
    pub fn new() ->Self {
        let m_map_sessions = Mutex::new(HashMap::default());
        Self {
            m_map_sessions
        }
    }

    pub async fn start(&self, addr:&String, fw_service:Arc<ServiceForward>) -> io::Result<()> {
        let listener: TcpListener = TcpListener::bind(addr).await.expect("service device listen failed");

        log::info!("[service-device]listen addr:{}", addr);

        let _ = tokio::spawn(async move{
            loop {
                let (socket, _) = listener.accept().await.unwrap();

                log::info!("[service-device]new connect addr:{:?}", socket.peer_addr());

                let fw_service = fw_service.clone();
                //todo: linux use tokio::uring 
                tokio::spawn(async move{

                    let (mut reader, writer) = socket.into_split();
                    let mut jt808_parse = Jt808DeserializeAndPackUp::new();

                    let sender_arc = Arc::new(Mutex::new(writer));
                    let mut buffer = bytes::BytesMut::with_capacity(8096);
                    let mut sessions: HashMap<String, Jt808Session> = HashMap::new();

                    loop {
                        match timeout(Duration::from_secs(60), reader.read_buf(&mut buffer)).await {
                            Ok(result) => {
                            let n = result.expect("[service-device]conn faield read");
                            if n > 0 {
                                    match jt808_parse.deserialize(&mut buffer) {
                                        Ok(package) => {
                                            if let Some(mut jtsub) = package {
                                                if let Some(jt808) = jtsub.get_first_jt() {
                                                    let sim = jt808.sim.to_string();

                                                    match sessions.get_mut(&sim) {
                                                        Some(session) => {
                                                            //是否已经closed
                                                            if session.is_closed() {
                                                                log::info!("[service-device]disconnect(session closed)");
                                                                break;
                                                            }
                                                            session.handle(&mut jtsub).await;
                                                        },
                                                        None => {
                                                            //获得转发列表
                                                            let fw_sender = ServiceForward::get_forward_sender(&fw_service, &sim).await;

                                                            let session_common: Arc<Jt808SessionShared> = Arc::new(Jt808SessionShared::new(
                                                                sender_arc.clone(),
                                                                JtPackage::new(jt808.sim.clone(), jt808.v19, jt808.ver, 1023),
                                                            ));
            
                                                            let mut session = Jt808Session::new(session_common.clone(),  fw_sender).await;
                                                            
                                                            session.handle(&mut jtsub).await;
            
                                                            sessions.insert(sim.clone(), session);
                                                            map_insert(sim.clone(), session_common);
                                                        },
                                                    }
                                                }
                                            }
                                        },
                                        Err(_err) => {
                                            log::info!("[service-device]disconnect(protocol)");
                                            break;
                                        },
                                    };
                                } else {
                                    log::info!("[service-device]disconnect(reason:net)");
                                    break;
                                }
                            },
                            Err(_) => {
                                log::info!("[service-device]disconnect(timeout)");
                                break;
                            },
                        };
                    }
                
                    for (sim, session) in sessions {
                        map_remove(&sim, session.session_shared);
                    }

                });
            }
        });

        Ok(())

    }
    
    pub async fn map_insert(&mut self, sim : String, value :Arc<Jt808SessionShared>) {        
        let mut map_senders = self.m_map_sessions.lock().await;
        if let Some(session_shared) = map_senders.insert(sim, value) {
            session_shared.close();
        }
    }
    
    pub async fn map_remove(&mut self, sim:&String, value:Arc<Jt808SessionShared>) {
        if value.is_closed() {
            return;
        }
        let mut map_senders = self.m_map_sessions.lock().await;
        if value.is_closed() {
            return;
        }
        
        if let Some(session_shared) = map_senders.remove(sim) {
            session_shared.close();
        }
    }
    
    pub async fn map_get(&self, sim : &String) -> Option<Arc<Jt808SessionShared>> {
        let mut map_senders = self.m_map_sessions.lock().await;
        let t = map_senders.get_mut(sim)?;
        Some(t.clone())
    }

}

impl GetSender for ServiceJt808  {
    fn get_sender(&self, sim:&String) -> Option<Arc<Jt808SessionShared>> {
        match map_get(sim) {
            Some(client_sender) => {
                Some(client_sender)
            },
            None => {
                None
            },
        }
    }
}

pub fn init() {
    *GLOBAL_DATA.lock().unwrap() = Some(HashMap::default());
}

pub async fn start(addr:&String, fw_service:Arc<ServiceForward>) -> io::Result<()> {
    
    let listener: TcpListener = TcpListener::bind(addr).await.expect("service device listen failed");

    log::info!("[service-device]listen addr:{}", addr);

    let _ = tokio::spawn(async move{
        loop {
            let (socket, _) = listener.accept().await.unwrap();

            log::info!("[service-device]new connect addr:{:?}", socket.peer_addr());

            let fw_service = fw_service.clone();
            //todo: linux use tokio::uring 
            tokio::spawn(async move{

                let (mut reader, writer) = socket.into_split();
                let mut jt808_parse = Jt808DeserializeAndPackUp::new();

                let sender_arc = Arc::new(Mutex::new(writer));
                let mut buffer = bytes::BytesMut::with_capacity(8096);
                let mut sessions: HashMap<String, Jt808Session> = HashMap::new();

                loop {
                    match timeout(Duration::from_secs(60), reader.read_buf(&mut buffer)).await {
                        Ok(result) => {
                           let n = result.expect("[service-device]conn faield read");
                           if n > 0 {
                                match jt808_parse.deserialize(&mut buffer) {
                                    Ok(package) => {
                                        if let Some(mut jtsub) = package {
                                            if let Some(jt808) = jtsub.get_first_jt() {
                                                let sim = jt808.sim.to_string();

                                                match sessions.get_mut(&sim) {
                                                    Some(session) => {
                                                        //是否已经closed
                                                        if session.is_closed() {
                                                            log::info!("[service-device]disconnect(session closed)");
                                                            break;
                                                        }
                                                        session.handle(&mut jtsub).await;
                                                    },
                                                    None => {
                                                        //获得转发列表
                                                        let fw_sender = ServiceForward::get_forward_sender(&fw_service, &sim).await;

                                                        let session_common: Arc<Jt808SessionShared> = Arc::new(Jt808SessionShared::new(
                                                            sender_arc.clone(),
                                                            JtPackage::new(jt808.sim.clone(), jt808.v19, jt808.ver, 1023),
                                                        ));
        
                                                        let mut session = Jt808Session::new(session_common.clone(),  fw_sender).await;
                                                        
                                                        session.handle(&mut jtsub).await;
        
                                                        sessions.insert(sim.clone(), session);
                                                        map_insert(sim.clone(), session_common);
                                                    },
                                                }
                                            }
                                        }
                                    },
                                    Err(_err) => {
                                        log::info!("[service-device]disconnect(protocol)");
                                        break;
                                    },
                                };
                            } else {
                                log::info!("[service-device]disconnect(reason:net)");
                                break;
                            }
                        },
                        Err(_) => {
                            log::info!("[service-device]disconnect(timeout)");
                            break;
                        },
                    };
                }
            
                for (sim, session) in sessions {
                    map_remove(&sim, session.session_shared);
                }

            });
        }
    });

    Ok(())
}

pub async fn get_sender(sim:&String) -> Option<Arc<Jt808SessionShared>> {
    match map_get(sim) {
        Some(client_sender) => {
            Some(client_sender)
        },
        None => {
            None
        },
    }
}

fn map_insert(sim : String, value :Arc<Jt808SessionShared>) {
    let mut binding = GLOBAL_DATA.lock().unwrap();
    let map_senders = binding.as_mut().unwrap();
    
    if let Some(session_shared) = map_senders.insert(sim, value) {
        session_shared.close();
    }
}

fn map_remove(sim:&String, value:Arc<Jt808SessionShared>) {
    if value.is_closed() {
        return;
    }

    let mut binding = GLOBAL_DATA.lock().unwrap();
    let map_senders = binding.as_mut().unwrap();

    if value.is_closed() {
        return;
    }
    
    if let Some(session_shared) = map_senders.remove(sim) {
        session_shared.close();
    }
}

fn map_get(sim : &String) -> Option<Arc<Jt808SessionShared>> {
    let mut binding = GLOBAL_DATA.lock().unwrap();
    let map_senders = binding.as_mut().unwrap();

    let t = map_senders.get_mut(sim)?;
    Some(t.clone())
}