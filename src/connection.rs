use std::task::{Context, Poll};

use crate::Frame;
use crate::codec::Codec;
pub struct Connection {
    rx: quinn::RecvStream,
    tx: quinn::SendStream,
}


pub struct ConnectionReceiver {
    rx: quinn::RecvStream,
    kind_buf: [u8; 1],
    size_buf: [u8; 8],
    state: ReadState,
}


pub enum ReadState {
    WaitKind,
    WaitSize,
    WaitPayload {
        bias: usize,
    },
}

impl ConnectionReceiver  {
    pub fn poll_read(&mut self, cx: &mut Context) -> Poll<Result<Frame, quinn::ReadError>> {
        loop {
            match self.state {
                ReadState::WaitKind => {
                    match self.rx.poll_read(cx, &mut self.kind_buf) {
                        Poll::Ready(Ok(size)) => {
                            self.state = ReadState::WaitSize;
                        }
                        Poll::Ready(Err(e)) => {
                            return Poll::Ready(Err(e));
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                },
                ReadState::WaitSize => {
                    match self.rx.poll_read(cx, &mut self.size_buf) {
                        Poll::Ready(Ok(size)) => {
                            self.state = ReadState::WaitPayload {
                                bias: 0,
                            };
                        }
                        Poll::Ready(Err(e)) => {
                            return Poll::Ready(Err(e));
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                },
                ReadState::WaitPayload => {
                    let size = u64::from_be_bytes(self.size_buf) as usize;
                    let mut payload = vec![0u8; size];
                    match self.rx.poll_read(cx, &mut payload) {
                        Poll::Ready(Ok(size)) => {
                            self.state = ReadState::WaitKind;
                            return Poll::Ready(Ok(Frame {
                                kind: self.kind_buf[0].into(),
                                payload: payload.into(),
                            }));
                        }
                        Poll::Ready(Err(e)) => {
                            return Poll::Ready(Err(e));
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}