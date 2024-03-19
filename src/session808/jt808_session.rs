use std::{sync::{Arc, atomic::{Ordering, AtomicI32, AtomicBool}}, collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use jt808::{models::{Jt0x0100, Jt0x0102, Jt0x8100, Jt0x0001, Jt808BodySerialize}, JtSubMerger, JtPackage};
use jt_util::bytes_gbk::BytesGBK;
use tokio::{net::tcp::OwnedWriteHalf, sync::{Mutex, Notify}, io::AsyncWriteExt, time::timeout};

use crate::{service_forward::ForwardSimSender, session_forward::forward_item::ForwardItem};

pub struct Jt808SessionShared {
    sender : Arc<Mutex<OwnedWriteHalf>>,
    package : JtPackage,
    fw_ids: Mutex<HashMap<u16, Arc<ForwardItem>>>,
    gw_ids: Mutex<HashMap<u16, (Arc<Notify>, Arc<AtomicI32>)>>,
    is_closed:AtomicBool
}

impl Jt808SessionShared {
    pub fn new(sender:Arc<Mutex<OwnedWriteHalf>>, package:JtPackage) -> Self {
        Jt808SessionShared {
            sender,
            package,
            fw_ids:Mutex::new(HashMap::new()),
            gw_ids:Mutex::new(HashMap::new()),
            is_closed:AtomicBool::new(false)
        }
    }

     //来自转发服务
    pub async fn forward_recv(&self, jtsub:&mut JtSubMerger, forward_item:&Arc<ForwardItem>) -> bool {

        if self.is_closed() {
            return false;
        }

        let mut sn = self.package.distribute_sn(jtsub.data.len() as u16);

        self.fw_ids.lock().await.insert(sn, forward_item.clone());

        for i in 0..jtsub.data.len() as u16 {
            let tt = jtsub.data.get(&i).unwrap();
            let data = tt.rec_modify_sn(sn);

            let _ = self.sender.lock().await.write_all(&data);
            sn+=1;
        }

        return true;
    }

    //来自http的发送
    pub async fn send_cmd<T: Jt808BodySerialize>(&self, id:u16, jtcmd:&mut T) -> i32 {

        //todo:sn package.serialize 返回
        let sn = self.package.get_sn();
        let buf = self.package.serialize(id, 0, jtcmd);
        let mut writer: tokio::sync::MutexGuard<'_, OwnedWriteHalf> = self.sender.lock().await;
        let _ = writer.write(&buf).await;

        let notify = Arc::new(Notify::new());
        let ret = Arc::new(AtomicI32::new(0));
        self.gw_ids.lock().await.insert(sn, (notify.clone(), ret.clone()));

        match timeout(std::time::Duration::from_secs(5), notify.notified()).await {
            Ok(_) => {
                return ret.load(Ordering::Relaxed);
            },
            Err(_) => {
                self.gw_ids.lock().await.remove(&sn);
                return -1;
            },
        }
    }

    pub fn is_closed(&self) -> bool
    {
        self.is_closed.load(Ordering::Relaxed)
    }

    pub fn close(&self)
    {
        self.is_closed.store(true, Ordering::Relaxed);
    }
}

pub struct Jt808Session {
    pub session_shared: Arc<Jt808SessionShared>,
    time_last_recv: u64,
    fw_sender:ForwardSimSender,
}

impl Jt808Session {
    pub async fn new(session_shared:Arc<Jt808SessionShared>, mut fw_sender:ForwardSimSender) -> Self {
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        fw_sender.bind_device(session_shared.clone()).await;

        Self {
            session_shared,
            time_last_recv:time_now,
            fw_sender
        }
    }
    
    pub async fn handle(&mut self, jtsub:&mut JtSubMerger) {

        let jt = jtsub.get_first_jt().unwrap();
        let sn: u16 = jt.sn;
        let id = jt.id;
        
        match jt.id {
            0x0001 => { //终端通用应答

                let jt0x0001 = jtsub.trans_body::<Jt0x0001>();
                let answer_sn = jt0x0001.answer_sn;
                let result = jt0x0001.result;

                //网关应答
                if let Some((notify, ret)) = self.session_shared.gw_ids.lock().await.remove(&answer_sn) {
                    ret.store(result as i32, Ordering::Relaxed);
                    notify.notify_one();
                    return;
                }

                //转发应答  todo:fw_ids添加超时
                if let Some(forward_item) = self.session_shared.fw_ids.lock().await.remove(&answer_sn) {
                    forward_item.forward_send(jtsub).await;
                    return;
                }
            },
            0x0002 => { //终端心跳
                self.time_last_recv = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            },
            0x0100 => { //终端注册
                let tt = jtsub.trans_body::<Jt0x0100>();
                
                log::info!("[service-device][session]recv 0x0100:{:?}", tt);

                let mut resp0x8100 = Jt0x8100 {
                    answer_sn: sn,
                    result: 0,
                    authority_code: BytesGBK::new_with_bytes(bytes::Bytes::from("9090980")),
                };
                                    
                let buf = self.session_shared.package.serialize(0x8100, 0, &mut resp0x8100);
                let mut writer: tokio::sync::MutexGuard<'_, OwnedWriteHalf> = self.session_shared.sender.lock().await;
                let _ = writer.write(&buf).await;

                log::info!("[service-device][session]response 0x8100:{:?}", resp0x8100);
                return;
            },
            0x0003 => { //终端注销
                

            }
            0x0102 => { //终端鉴权
                let tt = jtsub.trans_body::<Jt0x0102>();

                log::info!("[service-device][session]recv 0x0102:{:?}", tt);

                let mut resp0x8001 = Jt0x0001 {
                    answer_sn: sn,
                    answer_id: id,
                    result: 0,
                };

                let buf = self.session_shared.package.serialize(0x8001, 0, &mut resp0x8001);
                let mut writer = self.session_shared.sender.lock().await;
                let _ = writer.write(&buf).await;

                log::info!("[service-device][session]response 0x0102:{:?}", resp0x8001);
                return;
            },
            0x0104 => { //查询终端参数应答
                
            }
            0x0200 => { //gps

            }
            _ => {
            }
        }
        //全部转发
        self.forward_send(jtsub).await;
    }

    pub async fn forward_send(&mut self, jtsub:&mut JtSubMerger) {
        
        match jtsub.end() {
            Some(packages) => {
                for item in packages {
                    self.fw_sender.forward_send(&item.get_bytes()).await;
                }
            },
            None => {},
        }
    }

    pub fn is_closed(&self) -> bool {
        self.session_shared.is_closed()
    }
}