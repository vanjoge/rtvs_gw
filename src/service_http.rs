use std::{collections::HashMap, num::ParseIntError, sync::Arc};

use axum::{
    routing::get,
    Router, extract::Query,
};
use bytes::Bytes;
use jt1078::extend808::{Jt0x9101, Jt0x9102, Jt0x9201, Jt0x9202, Jt0x9205};
use jt808::{models::{Jt808, Jt808BodyTrans, Jt808BodySerialize}, bytes::JtBytes};

use crate::{service_device::{self, GetSender}, session808::jt808_session::Jt808SessionShared};


struct ServiceHttp {
    sender : Box<dyn GetSender>,
}

impl ServiceHttp {
    pub fn new(sender:Box<dyn GetSender>) ->Self {
        Self{
            sender
        }
    }

    pub async fn start(addr:&String) {
    
        let app = Router::new()
        .route("/api/VideoControl", get(root));
    
        log::info!("[service-http]listen addr:{}", addr);
    
        axum::Server::bind(&addr.parse().expect("Inavailable ip address"))
        .serve(app.into_make_service())
        .await
        .unwrap();
    }
    
    async fn root(Query(args): Query<HashMap<String, String>>) -> &'static str {
        log::info!("[service-http]Control args:{:?}", args);
        
        match args.get("Content") {
            Some(value) => {
                match decode_hex(value) {
                    Ok(mut result) => {
                        let mut jt808 = Jt808::from_http(&result);
                        let sim = jt808.sim.to_string();
                        let body = result.split_off(jt808.get_head_len() - 1);
                        match jt808.id {
                            0x9101 => {
                                let mut jt9101 = Jt0x9101::fill_new(&mut JtBytes::from(body), &mut jt808);
                                log::info!("[service-http]Control Jt9101:{:?}", jt9101);
    
                                return send_cmd(&sim, 0x9101, &mut jt9101).await;
                            },
                            0x9102 => {
                                let mut jt9102 = Jt0x9102::fill_new(&mut JtBytes::from(body), &mut jt808);
                                log::info!("[service-http]Control Jt9101:{:?}", jt9102);
    
                                return send_cmd(&sim, 0x9102, &mut jt9102).await;
                            },
                            0x9201 => {
                                let mut jt9201 = Jt0x9201::fill_new(&mut JtBytes::from(body), &mut jt808);
                                log::info!("[service-http]Control Jt9201:{:?}", jt9201);
    
                                return send_cmd(&sim, 0x9201, &mut jt9201).await;
                            },
                            0x9202 => {
                                let mut jt9202 = Jt0x9202::fill_new(&mut JtBytes::from(body), &mut jt808);
                                log::info!("[service-http]Control Jt9202:{:?}", jt9202);
    
                                return send_cmd(&sim, 0x9202, &mut jt9202).await;
                            },
                            0x9205 => {
                                let mut jt9205 = Jt0x9205::fill_new(&mut JtBytes::from(body), &mut jt808);
                                log::info!("[service-http]Control Jt9101:{:?}", jt9205);
    
                                return send_cmd(&sim, 0x9205, &mut jt9205).await;
                            },
                            _ => {
                                return "0"
                            }
                        }
    
                        
                    }
                    Err(_) => {
                        return "0";
                    },
                }
            },
            None => {
                return "0";
            },
        }
    }
    
    async fn send_cmd<T:Jt808BodySerialize>(self, sim:&String, id:u16, cmd:&mut T) -> &'static str {
        
        match self.sender.get_sender(sim) {
            Some(sender) => {
                sender.send_cmd(id, cmd).await;
                return "1";
            },
            None => {
                return "0";
            },
        }
    }
    
    fn decode_hex(s: &str) -> Result<Bytes, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }
    
}


pub async fn start(addr:&String) {
    
    let app = Router::new()
    .route("/api/VideoControl", get(root));

    log::info!("[service-http]listen addr:{}", addr);

    axum::Server::bind(&addr.parse().expect("Inavailable ip address"))
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn root(Query(args): Query<HashMap<String, String>>) -> &'static str {
    log::info!("[service-http]Control args:{:?}", args);
    
    match args.get("Content") {
        Some(value) => {
            match decode_hex(value) {
                Ok(mut result) => {
                    let mut jt808 = Jt808::from_http(&result);
                    let sim = jt808.sim.to_string();
                    let body = result.split_off(jt808.get_head_len() - 1);
                    match jt808.id {
                        0x9101 => {
                            let mut jt9101 = Jt0x9101::fill_new(&mut JtBytes::from(body), &mut jt808);
                            log::info!("[service-http]Control Jt9101:{:?}", jt9101);

                            return send_cmd(&sim, 0x9101, &mut jt9101).await;
                        },
                        0x9102 => {
                            let mut jt9102 = Jt0x9102::fill_new(&mut JtBytes::from(body), &mut jt808);
                            log::info!("[service-http]Control Jt9101:{:?}", jt9102);

                            return send_cmd(&sim, 0x9102, &mut jt9102).await;
                        },
                        0x9201 => {
                            let mut jt9201 = Jt0x9201::fill_new(&mut JtBytes::from(body), &mut jt808);
                            log::info!("[service-http]Control Jt9201:{:?}", jt9201);

                            return send_cmd(&sim, 0x9201, &mut jt9201).await;
                        },
                        0x9202 => {
                            let mut jt9202 = Jt0x9202::fill_new(&mut JtBytes::from(body), &mut jt808);
                            log::info!("[service-http]Control Jt9202:{:?}", jt9202);

                            return send_cmd(&sim, 0x9202, &mut jt9202).await;
                        },
                        0x9205 => {
                            let mut jt9205 = Jt0x9205::fill_new(&mut JtBytes::from(body), &mut jt808);
                            log::info!("[service-http]Control Jt9101:{:?}", jt9205);

                            return send_cmd(&sim, 0x9205, &mut jt9205).await;
                        },
                        _ => {
                            return "0"
                        }
                    }

                    
                }
                Err(_) => {
                    return "0";
                },
            }
        },
        None => {
            return "0";
        },
    }
}

async fn send_cmd<T:Jt808BodySerialize>(sim:&String, id:u16, cmd:&mut T) -> &'static str {
    match service_device::get_sender(&sim).await {
        Some(sender) => {
            sender.send_cmd(id, cmd).await;
            return "1";
        },
        None => {
            return "0";
        },
    }
}

fn decode_hex(s: &str) -> Result<Bytes, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
