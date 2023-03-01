use std::{io, sync::Arc, time::Duration};

use async_std::{channel, io::ReadExt, net::TcpStream, task};
use futures::future::BoxFuture;
use ruisutil::bytes::{ByteBox, ByteSteamBuf};

use crate::socks::msg::{self, tcps};

use super::{MessageRecv, Senders};

#[derive(Clone)]
pub struct Messager {
    inner: ruisutil::ArcMut<Inner>,
}

struct Inner {
    ctx: ruisutil::Context,
    conn: TcpStream,
    shuted: bool,
    is_serv: bool,
    //check
    ctms: ruisutil::Timer,
    ctmout: ruisutil::Timer,
    msgs_sx: channel::Sender<msg::Messages>,
    msgs_rx: channel::Receiver<msg::Messages>,

    buf: ByteSteamBuf,

    recver: Box<dyn MessageRecv + Send + Sync>,
}

impl Messager {
    pub fn new<T>(
        ctx: &ruisutil::Context,
        conn: TcpStream,
        recver: T,
        sndbufln: usize,
    ) -> (Self, Senders)
    where
        T: MessageRecv + Send + Sync + 'static,
    {
        let (sx, rx) = if sndbufln > 0 {
            channel::bounded::<msg::Messages>(sndbufln)
        } else {
            channel::unbounded::<msg::Messages>()
        };
        let c = Self {
            inner: ruisutil::ArcMut::new(Inner {
                ctx: ruisutil::Context::background(Some(ctx.clone())),
                conn: conn,
                shuted: false,
                is_serv: false,

                ctms: ruisutil::Timer::new(Duration::from_secs(20)),
                ctmout: ruisutil::Timer::new(Duration::from_secs(30)),
                msgs_sx: sx.clone(),
                msgs_rx: rx,

                buf: ByteSteamBuf::new(&ctx, 1024, Duration::from_millis(100)),

                recver: Box::new(recver),
            }),
        };
        (c, sx)
    }

    pub fn stop(&self) -> io::Result<()> {
        if self.inner.shuted {
            return Ok(());
        }
        println!("msger conn will stop");
        let ins = unsafe { self.inner.muts() };
        ins.shuted = true;
        self.inner.ctx.stop();
        self.inner.buf.close();
        self.inner.msgs_sx.close();
        ins.conn.shutdown(std::net::Shutdown::Both)
    }

    pub async fn run(&self, servs: bool, is_stream_buf: bool) {
        self.inner.ctmout.reset();
        unsafe { self.inner.muts().is_serv = servs };
        let c = self.clone();
        task::spawn(async move {
            c.run_send().await;
            c.inner.ctx.stop();
            println!("Messager run_send end!!");
        });
        if is_stream_buf {
            let c = self.clone();
            task::spawn(async move {
                if let Err(e) = c.run_read().await {
                    println!("run_read err:{}", e);
                    c.inner.ctx.stop();
                }
                println!("Messager run_recv end!!");
            });
            let c = self.clone();
            task::spawn(async move {
                if let Err(e) = c.run_parse().await {
                    println!("run_parse err:{}", e);
                    c.inner.ctx.stop();
                }
                println!("Messager run_recv end!!");
            });
        } else {
            let c = self.clone();
            task::spawn(async move {
                c.run_recv().await;
                println!("Messager run_recv end!!");
                c.inner.ctx.stop();
            });
        }
        println!("Messager start run check");
        while !self.inner.ctx.done() {
            self.run_check().await;
            task::sleep(Duration::from_millis(100)).await;
        }
        if let Err(e) = self.stop() {
            println!("Messager end stop err:{}", e);
        }
        println!("Messager end run check");
    }

    async fn run_read(&self) -> io::Result<()> {
        let ins = unsafe { self.inner.muts() };
        while !self.inner.ctx.done() {
            let mut buf = vec![0u8; 4096].into_boxed_slice();
            let n = ins.conn.read(&mut buf).await?;
            if n <= 0 {
                return Err(ruisutil::ioerr("read size=0 err!!", None));
            }
            let bts = ByteBox::new(Arc::new(buf), 0, n);
            self.inner.buf.push(bts).await?;
        }
        Ok(())
    }
    async fn run_parse(&self) -> io::Result<()> {
        while !self.inner.ctx.done() {
            let msg = tcps::parse_steam_msg(&self.inner.buf).await?;
            if let Err(e) = self.on_msg(msg).await {
                println!("run_parse on_msg err:{}", e);
            }
        }
        Ok(())
    }
    async fn on_msg(&self, msg: msg::Message) -> io::Result<()> {
        let ctrl = msg.control;
        match ctrl {
            0 => {
                self.inner.ctmout.reset();
                if self.inner.is_serv {
                    let msg = msg::Messages {
                        control: 0,
                        cmds: Some("heart".into()),
                        heads: None,
                        bodys: None,
                        bodybuf: None,
                    };
                    if let Err(e) = self.inner.msgs_sx.try_send(msg) {
                        println!("chan send err:{}", e);
                    }
                }
            }
            _ => {
                let c = self.clone();
                // let rc = self.inner.recver.clone();
                task::spawn(async move {
                    if let Err(e) = c.inner.recver.on_msg(msg).await {
                        println!("Messager recv on_msg (ctrl:{}) err:{}", ctrl, e);
                        if e.kind() == io::ErrorKind::Interrupted {
                            // let _ = c.stop();
                            c.inner.ctx.stop();
                            task::sleep(Duration::from_millis(200)).await;
                        }
                    }
                });
            }
        }
        Ok(())
    }
    async fn run_recv(&self) {
        let ins = unsafe { self.inner.muts() };
        while !self.inner.ctx.done() {
            match msg::tcps::parse_msg(&self.inner.ctx, &mut ins.conn).await {
                Err(e) => {
                    println!("Messager parse_msg err:{:?}", e);
                    // let _ = self.stop();
                    self.inner.ctx.stop();
                    task::sleep(Duration::from_millis(200)).await;
                }
                Ok(v) => {
                    if let Err(e) = self.on_msg(v).await {
                        println!("run_recv on_msg err:{}", e);
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
                    // let _ = self.stop();
                    self.inner.ctx.stop();
                    task::sleep(Duration::from_millis(200)).await;
                }
                Ok(v) => {
                    // println!("-------test-run_send: send_msgs start:ctrl={}", v.control);
                    if let Err(e) = msg::tcps::send_msgs(&self.inner.ctx, &mut ins.conn, v).await {
                        println!("run_send send_msgs err:{}", e);
                        task::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
    }
    async fn run_check(&self) {
        /* println!(
            "m run_check:ctmout={}ms!!!--------------",
            self.inner.ctmout.tmdur().as_millis()
        ); */
        if self.inner.ctmout.tmout() {
            // let _ = self.stop();
            println!("msger heart timeout!!");
            self.inner.ctx.stop();
            return;
        }

        if !self.inner.is_serv && self.inner.ctms.tick() {
            let msg = msg::Messages {
                control: 0,
                cmds: Some("heart".into()),
                heads: None,
                bodys: None,
                bodybuf: None,
            };
            if let Err(e) = self.inner.msgs_sx.try_send(msg) {
                println!("chan send err:{}", e);
            }
        }

        // let rc = self.inner.recver.clone();
        self.inner.recver.on_check().await;
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
