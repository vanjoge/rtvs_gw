use bytes::{BytesMut, Buf};
use jt808::{models::Jt808, JtSubMerger};
use jt_util::bytes_bcd::BytesBCD;

use crate::session808::jt808_parse::Jt808PackUp;


pub enum ForwardCodecError {
    /// 非转发封包格式
    NotForwardProtocol,
}

pub enum ReturnType {
    Cmd(u8, Vec<String>),
    Data(JtSubMerger)
}

pub struct ForwardParse {
    package_size:usize,
    jt808_packup:Jt808PackUp
}

impl ForwardParse {

    pub fn new() -> Self {
        ForwardParse {
            package_size: 0,
            jt808_packup: Jt808PackUp::new()
        }
    }

    pub fn parse(&mut self, buf: &mut BytesMut) -> Result<Option<ReturnType>, ForwardCodecError>  {
        
        if self.package_size == 0 {
            if buf.len() < 2 {
                return Ok(None);
            }
            self.package_size = buf.get_u16().into();
            if buf.len() < self.package_size {
                return Ok(None);
            }
        }

        if buf.len() < self.package_size {
            return Ok(None);
        }

        if buf[0] == 0xff && buf[1] == 0xff && buf[2] == 0xff {
            
            if buf[3] == 0x01 || buf[3] == 0x02 || buf[3] == 0x03 {
                let cmd_type = buf[3];
                buf.advance(4);

                let mut parse_size = 4;
                loop {
                    if buf.len() < 2 {
                        return Err(ForwardCodecError::NotForwardProtocol);
                    }
                    let sim_size_reading = buf.get_u16().into();
                    parse_size += 2;
                    if buf.len() < sim_size_reading {
                        return Err(ForwardCodecError::NotForwardProtocol);
                    }
                    let sim_bcd = buf.split_to(sim_size_reading);
                    if sim_size_reading == 0 {
                        return Err(ForwardCodecError::NotForwardProtocol);
                    }
                    parse_size += sim_size_reading;
        
                    let sim = BytesBCD::get_string(sim_bcd.into());
                    let mut list_sims = Vec::new();
                    list_sims.push(sim);

                    self.package_size = 0;
                    if parse_size == self.package_size {
                        return Ok(Some(ReturnType::Cmd(cmd_type, list_sims)));
                    } else if parse_size > self.package_size {
                        return Err(ForwardCodecError::NotForwardProtocol);
                    }
                }

            } else {
                buf.advance( self.package_size);
            }
        } else {
            let data = buf.split_to(self.package_size);
            let jt808: Jt808 = Jt808::from(data.freeze());

            match self.jt808_packup.get_sub_merger(jt808) {
                Some(jtsub) => {
                    self.package_size = 0;
                    return Ok(Some(ReturnType::Data(jtsub)));
                },
                None => {

                },
            }
        }

        self.package_size = 0;
        return Ok(None);
    }

}