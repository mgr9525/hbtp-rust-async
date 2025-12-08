use std::sync::Arc;

use ruisutil::bytes;

#[derive(Clone)]
pub struct Messages {
    pub control: i32,
    pub cmds: Option<String>,
    pub heads: Option<bytes::Bytes>,
    pub bodys: Option<bytes::Bytes>,
    pub bodybuf: Option<Arc<bytes::ByteBoxBuf>>,
}
#[derive(Clone)]
pub struct Message {
    pub version: u16,
    pub control: i32,
    pub cmds: String,
    pub heads: Option<bytes::Bytes>,
    pub bodys: MsgBody,
}

#[derive(Clone)]
pub enum MsgBody {
    None,
    Bytes(bytes::Bytes),
    BoxBuf(bytes::ByteBoxBuf),
}
impl MsgBody {
    pub fn len(&self) -> usize {
        match self {
            MsgBody::None => 0,
            MsgBody::Bytes(v) => v.len(),
            MsgBody::BoxBuf(v) => v.len(),
        }
    }
}
impl Message {
    pub fn new() -> Self {
        Self {
            version: 0,
            control: 0,
            cmds: String::new(),
            heads: None,
            bodys: MsgBody::None,
        }
    }
    pub fn own_bodys(&mut self) -> MsgBody {
        std::mem::replace(&mut self.bodys, MsgBody::None)
    }
    pub fn body_box(&self) -> Option<bytes::Bytes> {
        match &self.bodys {
            MsgBody::None => None,
            MsgBody::Bytes(v) => Some(v.clone()),
            MsgBody::BoxBuf(v) => Some(v.to_bytes()),
        }
    }
}

pub struct Messageus {
    pub control: i32,
    pub cmds: Option<String>,
    pub heads: Option<bytes::Bytes>,
    pub bodys: MsgBody,
}
pub struct Messageu {
    pub version: u16,
    pub control: i32,
    pub cmds: String,
    pub heads: Option<bytes::Bytes>,
    pub bodys: Option<bytes::Bytes>,
}
impl Messageu {
    pub fn new() -> Self {
        Self {
            version: 0,
            control: 0,
            cmds: String::new(),
            heads: None,
            bodys: None,
        }
    }
}

/* pub fn make_udp_packet_v1()->bytes::ByteBox{
  // let rt=bytes::ByteBox::
} */
