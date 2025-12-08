use std::{io, sync::Arc, time::Duration};

use ruisutil::asyncs::{make_channel, net::TcpStream, task, AsyncReadExt, Receiver, Sender};
use ruisutil::bytes::ByteSteamBuf;

use crate::socks::msg::{self, tcps};

use super::{Senders, TMessageRecv};

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
    msgs_sx: Sender<msg::Messages>,
    msgs_rx: Receiver<msg::Messages>,

    buf: ByteSteamBuf,

    recver: Box<TMessageRecv>,
}

impl Messager {
    pub fn new(
        ctx: &ruisutil::Context,
        conn: TcpStream,
        recver: Box<TMessageRecv>,
        sndbufln: usize,
    ) -> (Self, Senders) {
        let (sx, rx) = if sndbufln > 0 {
            make_channel(sndbufln)
        } else {
            make_channel(100)
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

                recver: recver,
            }),
        };
        (c, sx)
    }

    pub async fn stop(&self) -> io::Result<()> {
        if self.inner.shuted {
            return Ok(());
        }
        println!("msger conn will stop");
        let ins = unsafe { self.inner.muts() };
        ins.shuted = true;
        self.inner.ctx.stop();
        self.inner.buf.close();
        ruisutil::asyncs::close_channel_snd(&self.inner.msgs_sx);
        ruisutil::asyncs::tcp_shutdownw_ac(&mut ins.conn).await
    }

    pub async fn run(&self, servs: bool, is_stream_buf: bool) {
        self.inner.ctmout.reset();
        unsafe { self.inner.muts().is_serv = servs };
        let c = self.clone();
        task::spawn(async move {
            if let Err(e) = c.run_send().await {
                println!("run_send err:{}", e);
            }
            c.inner.ctx.stop();
            println!("Messager run_send end!!");
        });
        if is_stream_buf {
            let c = self.clone();
            task::spawn(async move {
                if let Err(e) = c.run_read().await {
                    println!("run_read err:{}", e);
                    // c.inner.ctx.stop();
                }
                c.inner.buf.close();
                println!("Messager run_recv end!!");
            });
            let c = self.clone();
            task::spawn(async move {
                if let Err(e) = c.run_parse().await {
                    println!("run_parse err:{}", e);
                }
                c.inner.ctx.stop();
                println!("Messager run_recv end!!");
            });
        } else {
            let c = self.clone();
            task::spawn(async move {
                if let Err(e) = c.run_recv().await {
                    println!("run_recv err:{}", e);
                }
                c.inner.ctx.stop();
                println!("Messager run_recv end!!");
            });
        }
        println!("Messager start run check");
        while !self.inner.ctx.done() {
            self.run_check().await;
            ruisutil::asyncs::sleep(Duration::from_millis(100)).await;
        }
        if let Err(e) = self.stop().await {
            println!("Messager end stop err:{}", e);
        }
        println!("Messager end run check");
    }

    async fn run_read(&self) -> io::Result<()> {
        let ins = unsafe { self.inner.muts() };
        loop {
            let mut buf = vec![0u8; 4096];
            let n = ruisutil::fut_tmout_ctxend0(&self.inner.ctx, ins.conn.read(&mut buf)).await?;
            if n <= 0 {
                return Err(ruisutil::ioerr("read size=0 err!!", None));
            }
            self.inner
                .buf
                .push(ruisutil::bytes::bytes_with_len(buf, n))
                .await?;
        }
    }
    async fn run_parse(&self) -> io::Result<()> {
        loop {
            let v = tcps::parse_steam_msg(&self.inner.ctx, &self.inner.buf).await?;
            if let Err(e) = self.on_msg(v).await {
                println!("run_parse on_msg err:{}", e);
            }
        }
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
                            ruisutil::asyncs::sleep(Duration::from_millis(200)).await;
                        }
                    }
                });
            }
        }
        Ok(())
    }
    async fn run_recv(&self) -> std::io::Result<()> {
        let ins = unsafe { self.inner.muts() };
        loop {
            let v = msg::tcps::parse_msg(&self.inner.ctx, &mut ins.conn).await?;
            if let Err(e) = self.on_msg(v).await {
                println!("run_recv on_msg err:{}", e);
            }
        }
    }
    async fn run_send(&self) -> std::io::Result<()> {
        let ins = unsafe { self.inner.muts() };
        loop {
            let v = ruisutil::fut_tmout_ctxend0(
                &self.inner.ctx,
                ruisutil::asyncs::channel_recv(&mut ins.msgs_rx),
            )
            .await?;
            // println!("-------test-run_send: send_msgs start:ctrl={}", v.control);
            if let Err(e) = msg::tcps::send_msgs(&self.inner.ctx, &mut ins.conn, v).await {
                println!("run_send send_msgs err:{}", e);
                ruisutil::asyncs::sleep(Duration::from_millis(10)).await;
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
            // self.inner.msgs_sx.try_send(msg);
            let c = self.clone();
            if let Err(e) = ruisutil::asyncs::timeouts(Duration::from_secs(3), async move {
                c.inner
                    .msgs_sx
                    .send(msg)
                    .await
                    .map_err(|e| ruisutil::ioerr("send err", None))
            })
            .await
            {
                println!("msger run_check send heart err:{}", e);
            }
        }

        // self.inner.recver.on_check().await;
        let c = self.clone();
        let _ = ruisutil::timeoutios(Duration::from_secs(5), async move {
            c.inner.recver.on_check().await;
            Ok(())
        })
        .await;
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
