pub mod broker;
pub mod codec;
pub mod connection;
pub mod listener;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorId {
    node: u32,
    inner: u32,
}

use std::collections::HashMap;

use bytes::Bytes;
use quinn::Endpoint;
use serde::{Deserialize, Serialize};

pub struct NodeTx {
    id: NodeId,
    router: TxRouter,
}

pub struct NodeRx {
    id: NodeId,
    router: Vec<quinn::RecvStream>,
}

impl NodeRx {
    
}

impl NodeTx {
    pub async fn scan_event(&mut self, frame: Frame) -> Result<(), quinn::ReadError> {
        Ok(())
    }
    pub async fn broadcast_event<'event, T>(
        &mut self,
        event: T,
    ) -> Result<(), quinn::ConnectionError>
    where
        T: Event<'event>,
    {
        let bytes = bincode::serialize(&event).unwrap();
        let frame = Frame::new(FrameKind::Event, bytes);
        self.broadcast_frame(frame).await.unwrap();
        Ok(())
    }

    pub async fn broadcast_frame(&mut self, frame: Frame) -> Result<(), quinn::WriteError> {
        for next_jump in self.router.table.values_mut() {
            let tx = &mut next_jump.tx;
            let size = (frame.payload.len() as u64).to_be_bytes();
            tx.write(&self.id.inner.to_be_bytes()).await?;
            tx.write(&[frame.kind as u8]).await?;
            tx.write(&size).await?;
            tx.write_chunk(frame.payload.clone()).await?;
        }
        Ok(())
    }
}
pub trait Event<'de>: Serialize + Deserialize<'de> {
    const CODE: &'static [u8];
}

pub struct TxRouter {
    table: HashMap<NodeId, NextJump>,
}

pub struct NextJump {
    tx: quinn::SendStream,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct NodeId {
    inner: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    kind: FrameKind,
    payload: Bytes,
}

impl Frame {
    pub fn new(kind: FrameKind, payload: impl Into<Bytes>) -> Self {
        Self {
            kind,
            payload: payload.into(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Close = 0,
    Open = 1,
    Event = 2,

}

impl From<u8> for FrameKind {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Event,
            _ => panic!("Invalid FrameKind"),
        }
    }
}