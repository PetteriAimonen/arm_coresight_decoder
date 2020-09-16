//! Packet types for ARM Trace Port Interface Unit,
//! also called Trace Formatter.

#[derive(Debug, Eq, PartialEq)]
pub enum TPIUPacket {
    /// Full frame synchronization packet, emitted between frames.
    FrameSynchronization,

    /// Halfword synchronization packet, between or within frames.
    HalfwordSynchronization,

    /// Payload data
    Data(TraceSourceID, Vec<u8>),

    // Trigger event
    Trigger(Vec<u8>),

    /// Null source data, i.e. unused padding
    Null(Vec<u8>),

    // Undefined packet types
    Reserved(Vec<u8>),
    Invalid(String),
}

/// Represents trace source ID
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraceSourceID(pub u8);

impl TraceSourceID {
    pub fn to_packet(&self, data: Vec<u8>) -> TPIUPacket {
        match self.0 {
            0x00 => TPIUPacket::Null(data),
            0x01 ... 0x6F => TPIUPacket::Data(*self, data),
            0x7D => TPIUPacket::Trigger(data),
            0x7F => TPIUPacket::Invalid(String::from("TraceSourceID 0x7F is invalid")),
            _ => TPIUPacket::Reserved(data),
        }
    }
}