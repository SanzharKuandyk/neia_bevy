use postcard::Error as PostcardError;
use serde::{Deserialize, Serialize};

/// Core message trait for client-server communication.
///
/// Every message type (system or game) must implement this trait.
/// Messages are serialized with postcard and framed with a name prefix on the wire.
pub trait Message: Send + Sync + 'static + Serialize + for<'de> Deserialize<'de> {
    /// Unique name identifying this message type (e.g., "sys.ready", "game.move").
    fn name(&self) -> &'static str;

    fn to_bytes(&self) -> Result<Vec<u8>, PostcardError> {
        postcard::to_allocvec(self)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, PostcardError>
    where
        Self: Sized,
    {
        postcard::from_bytes(bytes)
    }
}

/// Encode a message into wire format: [u16 name_len LE][name bytes][payload bytes]
pub fn encode_message<M: Message>(msg: &M) -> Result<Vec<u8>, PostcardError> {
    let name = msg.name().as_bytes();
    let name_len = name.len() as u16;

    let mut buf = Vec::with_capacity((2 + name_len as usize) + 128);
    buf.extend_from_slice(&name_len.to_le_bytes());
    buf.extend_from_slice(name);
    buf.extend_from_slice(&msg.to_bytes()?);
    Ok(buf)
}

/// Parse wire format and extract (name, payload) slices.
/// Returns None if the frame is malformed.
pub fn split_message_header(bytes: &[u8]) -> Option<(&str, &[u8])> {
    if bytes.len() < 2 {
        return None;
    }
    let name_len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    if bytes.len() < 2 + name_len {
        return None;
    }
    let name = std::str::from_utf8(&bytes[2..2 + name_len]).ok()?;
    let payload = &bytes[2 + name_len..];
    Some((name, payload))
}
