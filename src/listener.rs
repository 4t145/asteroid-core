use std::{collections::HashMap, net::SocketAddr, pin::Pin};

use bytes::Bytes;
use flume::select;
use quinn::{Runtime, VarInt};

use crate::{codec::Codec, connection, Frame};

pub struct Node<R> {
    ep: quinn::Endpoint,
    size: usize,
    rt: R,
    outbound_event: flume::Receiver<Bytes>,
}

pub enum FrameOutboundSideEvent {
    SendEvent {
        event: Bytes,
    },
    Close {
        reason: VarInt,
        addr: SocketAddr,
    },
    Open {
        addr: SocketAddr,
        tx: quinn::SendStream,
    },
}

impl<R> Node<R>
where
    R: Runtime,
{
    pub async fn run(self) {
        let (event_fan_out, event_fan_in) = flume::bounded::<Bytes>(self.size);
        let (ob_evt_send, ob_evt_recv) = flume::bounded::<FrameOutboundSideEvent>(self.size);
        {
            let event_fan_out = event_fan_out.clone();
            let ob_evt_send = ob_evt_send.clone();
            self.rt.spawn(Box::pin(async move {
                while let Ok(event) = self.outbound_event.recv_async().await {
                    event_fan_out.send(event.clone());
                    ob_evt_send
                        .send(FrameOutboundSideEvent::SendEvent { event })
                        .unwrap();
                }
            }));
        }
        {
            self.rt.spawn(Box::pin(async move {
                let mut tx_map = HashMap::new();
                while let Ok(event) = ob_evt_recv.recv_async().await {
                    match event {
                        FrameOutboundSideEvent::SendEvent { event } => {
                            let frame = Frame::new(crate::FrameKind::Event, event);
                            for tx in tx_map.values_mut() {
                                frame.clone().encode(tx).await.unwrap();
                            }
                        }
                        FrameOutboundSideEvent::Close { reason, addr } => {
                            tx_map.remove(&addr);
                        }
                        FrameOutboundSideEvent::Open { addr, tx } => {
                            tx_map.insert(addr, tx);
                        }
                    }
                }
            }));
        }
        while let Some(incoming) = self.ep.accept().await {
            let remote = incoming.remote_address();
            if let Ok(connection) = incoming.await {
                let (tx, mut rx) = connection.accept_bi().await.unwrap();
                let open = Frame::decode(&mut rx).await.unwrap();
                if open.kind != crate::FrameKind::Open {
                    todo!("invalid frame kind");
                }
                ob_evt_send
                    .send(FrameOutboundSideEvent::Open { addr: remote, tx })
                    .unwrap();
                let event_fan_out = event_fan_out.clone();
                self.rt.spawn(Box::pin(async move {
                    loop {
                        let frame = Frame::decode(&mut rx).await;
                        let frame = match frame {
                            Ok(frame) => frame,
                            Err(e) => {
                                break;
                            }
                        };
                        match frame.kind {
                            crate::FrameKind::Close => {
                                let close_result = rx.stop(VarInt::from_u32(0));

                                break;
                            }
                            crate::FrameKind::Open => {
                                todo!("invalid frame kind");
                            }
                            crate::FrameKind::Event => {
                                let bytes = frame.payload;
                                if let Err(e) = event_fan_out.send_async(bytes).await {
                                    break;
                                }
                            }
                        }
                    }
                }))
            }
        }
    }
}
