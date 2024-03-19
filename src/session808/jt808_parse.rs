use std::collections::HashMap;

use bytes::{BytesMut, BufMut, Buf};
use jt808::{models::Jt808, codec::Jt808CodecError, JtSubMerger};

pub struct Jt808Deserialize {
    /// 下一个检查索引
    next_index: usize,
    /// 允许检查最大长度
    max_length: usize,
    /// 是否抛弃到当前检查位置的数据
    is_discarding: bool,
    /// 已找到第一个7E
    find_0x7e: bool,
    /// 找到的第一个7E索引
    index_0x7e: usize,
    /// 包信息
    index_0x7d: Vec<usize>,
}

impl Jt808Deserialize {

    pub fn new() -> Self {
        Jt808Deserialize {
            next_index: 0,
            max_length: 1024,
            is_discarding: false,
            find_0x7e: false,
            index_0x7e: 0,
            index_0x7d: Vec::new(),
        }
    }

    pub fn deserialize(&mut self, buf: &mut BytesMut) -> Result<Option<Jt808>, Jt808CodecError>  {

        loop {
            
            let read_to = std::cmp::min(self.max_length.saturating_add(1), buf.len());

            let find_0x7e = buf[self.next_index..read_to]
            .iter()
            .enumerate()
            .position(|(i, b)| {
                if *b == 0x7du8 {
                    self.index_0x7d.push(self.next_index + i);
                    false
                } else {
                    *b == 0x7eu8
                }
            });

            match (self.is_discarding, find_0x7e) {
                (true, Some(offset)) => {
                    buf.advance(offset + self.next_index);
                    self.is_discarding = false;
                    self.index_0x7d.clear();

                    //当前作为起始标记
                    self.find_0x7e = true;
                    self.index_0x7e = offset;
                    self.next_index = 1;
                }
                (true, None) => {
                    //直接删除所有
                    buf.advance(read_to);
                    self.index_0x7d.clear();
                    self.find_0x7e = false;
                    self.index_0x7e = 0;
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(offset)) => {
                    // 找到7E
                    if self.find_0x7e {
                        self.find_0x7e = false;
                        let newline_index = offset + self.next_index;
                        if newline_index < 10 {
                            //位数不足时将结尾7E做开始再寻找
                            buf.advance(newline_index);
                            self.is_discarding = true;
                            return Err(Jt808CodecError::No808);
                        }
                        let package = buf.split_to(newline_index + 1);
                        self.next_index = 0;

                        return Jt808Deserialize::trans(&mut self.index_0x7d, package);
                    } else {
                        self.find_0x7e = true;
                        self.index_0x7e = offset;
                        self.next_index = offset + 1;
                    }
                }
                (false, None) if buf.len() > self.max_length => {
                    // Reached the maximum length without finding a
                    // newline, return an error and start discarding on the
                    // next call.
                    self.is_discarding = true;
                    return Err(Jt808CodecError::MaxLineLengthExceeded);
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the next
                    // call will resume searching at the current offset.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }    

    fn trans(index_0x7d: &mut Vec<usize>, mut package: BytesMut) -> Result<Option<Jt808>, Jt808CodecError>  {

        if index_0x7d.len() > 0 {

            let size = package.len() + index_0x7d.len();

            let mut bufnew = BytesMut::with_capacity(size);
            let mut newidx = 0;

            for idx in index_0x7d.iter() {
                newidx = idx - newidx;
                bufnew.put(package.split_to(newidx));
                if package[1] == 0x1 {
                    bufnew.put_u8(0x7d);
                } else if package[1] == 0x2 {
                    bufnew.put_u8(0x7e);
                } else {
                    return Err(Jt808CodecError::No808);
                }
                package.advance(2);
                newidx += 2;
            }
            if package.len() > 0 {
                bufnew.put(package);
            }
            return Ok(Some(Jt808::from(bufnew.freeze())));
        } else {
            return Ok(Some(Jt808::from(package.freeze())));
        }

    }

}

pub struct Jt808PackUp {
    all_packdata: HashMap<u16, JtSubMerger>,
}

impl Jt808PackUp {

    pub fn new() -> Self {
        Jt808PackUp { all_packdata: HashMap::new() }
    }

    pub fn get_sub_merger(&mut self, jt: Jt808) -> Option<JtSubMerger> {
        // 拼分包
        if let (Some(i), Some(sum)) = (jt.package_index, jt.package_total) {
            let first_sn = sum - i + 1;
            match self.all_packdata.get_mut(&first_sn) {
                Some(jtsub) => {
                    jtsub.add(jt);
                    if jtsub.check_pack() {
                        return self.all_packdata.remove(&first_sn);
                    }
                }
                None => {
                    self.all_packdata.insert(first_sn, JtSubMerger::add_new(first_sn, jt));
                }
            }
        } else {
            return Some(JtSubMerger::add_new(jt.sn, jt));
        }
        None
    }
}

pub struct Jt808DeserializeAndPackUp {
    jt808_deserialize:Jt808Deserialize,
    jt808_packup:Jt808PackUp
}

impl Jt808DeserializeAndPackUp {
    pub fn new() -> Self{
        Jt808DeserializeAndPackUp { 
            jt808_deserialize: Jt808Deserialize::new(), 
            jt808_packup: Jt808PackUp::new()
        }
    }

    pub fn deserialize(&mut self, buf: &mut BytesMut) -> Result<Option<JtSubMerger>, Jt808CodecError> {
        match self.jt808_deserialize.deserialize(buf) {
            Ok(package) => {
                match package {
                    None => {
                        return Ok(None);
                    }
                    Some(jt808) => {
                        return Ok(self.jt808_packup.get_sub_merger(jt808));
                    },
                }
            },
            Err(err) => {
                return Err(err);
            },
        }
    }
}




#[test]
fn test_bytes()
{
    //let buf = bytes::Bytes::from("12345678");
    //let t1 = buf[0];
    //let t2 = buf.get(0..2);
    //let t3 = buf[1];

    // let t1 = buf.get_u8();
    // let t2 = buf.get_u8();
    // let t3 = buf.get_u8();
    // let t4 = buf.get_u8();
}