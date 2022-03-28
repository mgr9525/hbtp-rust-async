use std::{io, time::Duration};

use async_std::{channel, net::TcpStream, task};
use futures::future::BoxFuture;

use super::msg;

pub type Senders = channel::Sender<msg::Messages>;

#[derive(Clone)]
pub struct Messager<T: MessageRecv + Clone + Sync + Send + 'static> {
    inner: ruisutil::ArcMut<Inner<T>>,
}

struct Inner<T: MessageRecv + Clone> {
    ctx: ruisutil::Context,
    conn: TcpStream,
    shuted: bool,
    //check
    ctms: ruisutil::Timer,
    ctmout: ruisutil::Timer,
    msgs_sx: channel::Sender<msg::Messages>,
    msgs_rx: channel::Receiver<msg::Messages>,

    recver: T,
}

impl<T: MessageRecv + Clone + Sync + Send + 'static> Messager<T> {
    pub fn new(
        ctx: &ruisutil::Context,
        conn: TcpStream,
        recver: &T,
        sndbufln: usize,
    ) -> (Self, Senders) {
        let (sx, rx) = if sndbufln > 0 {
            channel::bounded::<msg::Messages>(sndbufln)
        } else {
            channel::unbounded::<msg::Messages>()
        };
        let c = Self {
            inner: ruisutil::ArcMut::new(Inner {
                ctx: ctx.clone(),
                conn: conn,
                shuted: false,

                ctms: ruisutil::Timer::new(Duration::from_secs(20)),
                ctmout: ruisutil::Timer::new(Duration::from_secs(30)),
                msgs_sx: sx.clone(),
                msgs_rx: rx,

                recver: recver.clone(),
            }),
        };
        (c, sx)
    }

    pub fn stop(&self) -> io::Result<()> {
        if self.inner.shuted {
            return Ok(());
        }
        self.inner.ctx.stop();
        self.inner.msgs_sx.close();
        let ins = unsafe { self.inner.muts() };
        ins.shuted = true;
        ins.conn.shutdown(std::net::Shutdown::Both)
    }

    pub async fn run(&self) {
        self.inner.ctmout.reset();
        let c = self.clone();
        task::spawn(async move {
            c.run_send().await;
            println!("Messager run_send end!!");
        });
        let c = self.clone();
        task::spawn(async move {
            c.run_recv().await;
            println!("Messager run_recv end!!");
        });
        println!("Messager start run check");
        while !self.inner.ctx.done() {
            self.run_check().await;
            task::sleep(Duration::from_millis(100)).await;
        }
        let _ = self.stop();
        println!("Messager end run check");
    }

    pub async fn run_recv(&self) {
        let ins = unsafe { self.inner.muts() };
        while !self.inner.ctx.done() {
            match msg::parse_msg(&self.inner.ctx, &mut ins.conn).await {
                Err(e) => {
                    println!("Messager parse_msg err:{:?}", e);
                    let _ = self.stop();
                    task::sleep(Duration::from_millis(100)).await;
                }
                Ok(v) => {
                    let ctrl = v.control;
                    match ctrl {
                        0 => {
                            self.inner.ctmout.reset();
                            println!("remote reply heart");
                        }
                        _ => {
                            let c = self.clone();
                            let rc = self.inner.recver.clone();
                            task::spawn(async move {
                                if let Err(e) = rc.on_msg(v).await {
                                    println!("Messager recv on_msg (ctrl:{}) err:{}", ctrl, e);
                                    if e.kind() == io::ErrorKind::Interrupted {
                                        let _ = c.stop();
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    }
    async fn run_send(&self) {
        let ins = unsafe { self.inner.muts() };
        while !self.inner.ctx.done() {
            match self.inner.msgs_rx.recv().await {
                Err(e) => {
                    println!("run_send chan recv err:{}", e);
                    let _ = self.stop();
                    task::sleep(Duration::from_millis(100)).await;
                }
                Ok(v) => {
                    if let Err(e) = msg::send_msgs(&self.inner.ctx, &mut ins.conn, v).await {
                        println!("run_send send_msgs err:{}", e);
                        task::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
    }
    async fn run_check(&self) {
        if self.inner.ctmout.tick() {
            let _ = self.stop();
        }

        if self.inner.ctms.tick() {
            let msg = msg::Messages {
                control: 0,
                cmds: Some("heart".into()),
                heads: None,
                bodys: None,
                bodybuf: None,
            };
            if let Err(e) = self.inner.msgs_sx.send(msg).await {
                println!("chan send err:{}", e);
            }
        }

        let rc = self.inner.recver.clone();
        rc.on_check().await;
    }

    pub async fn send(&self, mv: msg::Messages) -> io::Result<()> {
        if let Err(e) = self.inner.msgs_sx.send(mv).await {
            //println!("chan send err:{}", e);
            Err(ruisutil::ioerr(format!("chan send err:{}", e), None))
        } else {
            Ok(())
        }
    }
}

pub trait MessageRecv: Clone {
    fn on_check(self) -> BoxFuture<'static, ()>;
    fn on_msg(self, msg: msg::Message) -> BoxFuture<'static, io::Result<()>>;
    // fn on_msg(self, msg: msg::Message) -> Pin<Box<dyn Future<Output = ()> + Sync + Send+'static>>;
}
