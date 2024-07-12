mod msger;
// mod msgbuf;

use ruisutil::asyncs::Sender;
pub use msger::Messager;
// pub use msgbuf::MessagBuffer;

use super::msg::{Message, Messages};
use ruisutil::asyncs::BoxFuture;

pub type Senders = Sender<Messages>;

pub trait MessageRecv {
    fn on_check(&self) -> BoxFuture<'static, ()>;
    fn on_msg(&self, msg: Message) -> BoxFuture<'static, std::io::Result<()>>;
    // fn on_msg(self, msg: msg::Message) -> Pin<Box<dyn Future<Output = ()> + Sync + Send+'static>>;
}
pub type TMessageRecv = dyn MessageRecv + Send + Sync;
