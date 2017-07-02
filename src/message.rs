use std::fmt::Debug;
use std::io::Result;
use std::usize;

use bytes::{
    Bytes,
    BytesMut,
};

use encoding::{
    decode_varint,
    encode_varint,
    invalid_input,
};

/// A Protocol Buffers message.
pub trait Message: Debug + PartialEq + Send + Sync {

    /// Encodes the message to the buffer.
    fn encode(&self, buf: &mut BytesMut);

    /// Encodes the message with a length-delimiting prefix to the buffer.
    fn encode_length_delimited(&self, buf: &mut BytesMut) {
        let len = self.encoded_len();
        buf.reserve(len);
        encode_varint(len as u64, buf);
        self.encode(buf);
    }

    /// Decodes an instance of the message from the buffer.
    /// The entire buffer will be consumed.
    fn decode(buf: &mut Bytes) -> Result<Self> where Self: Default {
        let mut message = Self::default();
        message.merge(buf)?;
        Ok(message)
    }

    /// Decodes a length-delimited instance of the message from the buffer.
    fn decode_length_delimited(buf: &mut Bytes) -> Result<Self> where Self: Default {
        let mut message = Self::default();
        message.merge_length_delimited(buf)?;
        Ok(message)
    }

    /// Decodes an instance of the message from the buffer, and merges it into
    /// `self`. The entire buffer will be consumed.
    fn merge(&mut self, buf: &mut Bytes) -> Result<()>;

    /// Decodes a length-delimited instance of the message from the buffer, and
    /// merges it into `self`.
    fn merge_length_delimited(&mut self, buf: &mut Bytes) -> Result<()> {
        let len = decode_varint(buf)?;
        if len > buf.len() as u64 {
            return Err(invalid_input("failed to merge message: buffer underflow"));
        }
        self.merge(&mut buf.split_to(len as usize))
    }

    /// Returns the encoded length of the message without a delimiter.
    fn encoded_len(&self) -> usize;
}

/*
impl <M> Message for Box<M> where M: Message {
    #[inline]
    fn encode_raw<B>(&self, buf: &mut BytesMut) {
        (**self).encode_raw(buf)
    }
    #[inline]
    fn merge<B>(&mut self, buf: &mut Take<B>) -> Result<()> where B: Buf {
        (**self).merge(buf)
    }
    #[inline]
    fn encoded_len(&self) -> usize {
        (**self).encoded_len()
    }
}
*/
