use std::any::{Any, TypeId};

use bytes::Bytes;
use serde::Serialize;

use crate::ActorId;

/// Transpose the message from another node's actors to local actors
pub struct Broker {
    connection_kind: ConnectionKind,
}

pub struct TypedBytes {
    type_id: (u64, u64),
    bytes: Bytes
}

pub struct MessageFrame {
    target: ActorId,
    type_id: (u64, u64),
    bytes: Bytes
}

impl Broker {
    pub async fn send<'w, T>(&mut self, mf: MessageFrame) 
    where T: Serialize + Any
    {
        if let ConnectionKind::Quic { rx, tx } = &mut self.connection_kind {
            let x = tx.write(&[]).await;
            
        }
    }
}

pub enum ConnectionKind {
    Quic { rx: quinn::RecvStream, tx: quinn::SendStream },
}
