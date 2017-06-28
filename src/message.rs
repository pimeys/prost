use std::collections::VecDeque;
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
    encoded_len_varint,
    invalid_input,
};

/// A Protocol Buffers message.
pub trait Message: Debug + PartialEq + Send + Sync {

    /// Encodes the message to the buffer. An error will be returned if the
    /// buffer has insuficient capacity .
    fn encode(&self, buf: &mut BytesMut) {
        let mut queue = VecDeque::new();
        buf.reserve(self.encoded_len_with_queue(&mut queue));
        self.encode_with_queue(&mut queue, buf)
    }

    /// Encodes the message, and writes it with a length-delimiter prefix to
    /// the buffer. An error will be returned if the buffer does not have
    /// sufficient capacity.
    fn encode_length_delimited(&self, buf: &mut BytesMut) {
        let mut queue = VecDeque::new();
        let len = self.encoded_len_with_queue(&mut queue);
        buf.reserve(len + encoded_len_varint(len as u64));
        encode_varint(len as u64, buf);
        self.encode_with_queue(&mut queue, buf);
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
        if len > buf.remaining() as u64 {
            return Err(invalid_input("failed to merge message: buffer underflow"));
        }
        self.merge(&mut buf.split_to(len as usize))
    }

    /// Returns the encoded length of the message without a delimiter.
    fn encoded_len(&self) -> usize {
        let mut queue = VecDeque::new();
        self.encoded_len_with_queu(&mut queue)
    }

    /// Encodes the message into the buffer.
    ///
    /// Lengths of nested messages (if any) are popped from the queue in post-order.
    #[doc(hidden)]
    fn encode_with_queue(&self, queue: &mut VecDeque<usize>, buf: &mut BytesMut);

    /// Returns the encoded length of the message without a delimiter.
    ///
    /// Lengths of nested messages (if any) are be pushed on to the queue in post-order.
    #[doc(hidden)]
    fn encoded_len_with_queue(&self, queue: &mut VecDeque<usize>) -> usize;
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
