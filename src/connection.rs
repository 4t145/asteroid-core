use bytes::Bytes;
use futures_core::future;
use quinn::VarInt;
use std::collections::VecDeque;
use std::future::poll_fn;
use std::sync::mpsc;
use std::sync::Mutex;
use std::task::{Context, Poll};

use crate::codec::Codec;
use crate::Frame;

pub struct ConnectionReceiver {
    rx: quinn::RecvStream,
    event_fan_out: flume::Sender<Bytes>,
}

impl ConnectionReceiver {
    pub async fn run_fun_out(&mut self) -> Result<(), quinn::ReadExactError> {
        loop {
            let frame = Frame::decode(&mut self.rx).await?;
            match frame.kind {
                crate::FrameKind::Close => {
                    let close_result = self.rx.stop(VarInt::from_u32(0));
                    break Ok(());
                }
                crate::FrameKind::Open => {
                    todo!("invalid frame kind");
                }
                crate::FrameKind::Event => {
                    let bytes = frame.payload;
                    if let Err(e) = self.event_fan_out.send_async(bytes).await {
                        break Ok(());
                    }
                }
            }
        }
    }
}

pub struct ConnectionSender {
    tx: quinn::SendStream,
}

pub struct Connection {
    sender: ConnectionSender,
    receiver: ConnectionReceiver,
}

impl Connection {
    pub async fn send_event(&mut self, event: Bytes) -> Result<(), quinn::WriteError> {
        let frame = Frame::new(crate::FrameKind::Event, event);
        frame.encode(&mut self.sender.tx).await?;
        Ok(())
    }
}
