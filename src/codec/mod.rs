pub trait Codec: Sized {
    async fn encode(self, stream: &mut quinn::SendStream) -> Result<(), quinn::WriteError>;
    async fn decode(stream: &mut quinn::RecvStream) -> Result<Self, quinn::ReadExactError>;
}

impl Codec for crate::Frame {
    async fn encode(self, stream: &mut quinn::SendStream) -> Result<(), quinn::WriteError> {
        let size = (self.payload.len() as u64).to_be_bytes();
        stream.write(&[self.kind as u8]).await?;
        stream.write(&size).await?;
        stream.write_chunk(self.payload).await?;
        Ok(())
    }
    async fn decode(stream: &mut quinn::RecvStream) -> Result<Self, quinn::ReadExactError> {
        let mut kind_buf = [0u8; 1];
        let mut size_buf = [0u8; 8];
        stream.read_exact(&mut kind_buf).await?;
        stream.read_exact(&mut size_buf).await?;
        let kind = kind_buf[0];
        let size = u64::from_be_bytes(size_buf) as usize;
        let mut payload = vec![0u8; size];
        stream.read_exact(&mut payload).await?;
        Ok(Self {
            kind: kind.into(),
            payload: payload.into(),
        })
    }
}
