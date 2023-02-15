
mod msger;
// mod msgbuf;

use async_std::channel;
pub use msger::Messager;
// pub use msgbuf::MessagBuffer;

use super::msg::{Message,Messages};
use futures::future::BoxFuture;


pub type Senders = channel::Sender<Messages>;

pub trait MessageRecv {
  fn on_check(&self) -> BoxFuture<'static, ()>;
  fn on_msg(&self, msg: Message) -> BoxFuture<'static, std::io::Result<()>>;
  // fn on_msg(self, msg: msg::Message) -> Pin<Box<dyn Future<Output = ()> + Sync + Send+'static>>;
}