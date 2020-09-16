//! Packet types for ARM Instrumentation Trace Macroblock.
//! Reference: ARMv7-M Architecture Reference Manual

use std::fmt;

/// Supported ITM packet types.
#[derive(Debug, Eq, PartialEq)]
pub enum ITMPacket {
    /// This packet is sent periodically for
    /// synchronizing hardware to byte boundaries.
    Synchronization,

    /// Indicates that data has been dropped due to buffer overflow.
    Overflow,

    /// Gives the timestamp delta after previous LocalTimestamp.
    LocalTimestamp(TimestampSync, LocalTimestampDelta),

    /// Global timestamp update
    GlobalTimestamp(GlobalTimestampValue),

    /// Software instrumentation port page number
    SoftwarePageNumber(InstrumentationPort),

    /// Software generated instrumentation message.
    Software(InstrumentationPort, DataValue),

    /// One or more event counters have wrapped back to zero.
    EventCounter(EventCounterFlags),

    /// Periodic program counter value reporting.
    ProgramCounter(Address),
    
    /// Periodic sampling packet, when processor is in sleep mode
    SleepMode,
    
    /// Exception (interrupt handler) entry/exit.
    Exception(ExceptionEvent, ExceptionNumber),
    
    /// Program counter value for the instruction that triggered
    /// data watchpoint.
    DataTracePC(ComparatorIndex, Address),
    
    /// Address offset inside a watchpoint range comparer.
    DataTraceOffset(ComparatorIndex, Address),
    
    /// Value of data read from memory.
    DataTraceReadData(ComparatorIndex, DataValue),
    
    /// Value of data written to memory.
    DataTraceWriteData(ComparatorIndex, DataValue),
    
    /// Extended information about a source
    Extension(ExtendedInformation),

    /// Undefined packet types
    Reserved(u8),
    Invalid(String),
}

/// Represents a memory address on the target processor.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Address(pub u32);

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

/// Represents an exception/interrupt number
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExceptionNumber(pub u32);

/// Represents a watchpoint comparator index
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComparatorIndex(pub u32);

/// Represents software instrumentation port number
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct InstrumentationPort(pub u32);

/// Represents local timestamp delta value
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalTimestampDelta(pub u32);

/// Represents global timestamp value.
/// known_mask identifies which bits of the timestamp are valid.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct GlobalTimestampValue {
    pub timestamp: u64,
    pub known_mask: u64,
    pub wrap: bool,
    pub clock_change: bool
}

/// A variably sized data value, corresponding to bus access size.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum DataValue {
    U8(u8), U16(u16), U32(u32)
}

impl DataValue {
    pub fn to_u32(&self) -> u32 {
        match *self {
            DataValue::U8(byte) => byte as u32,
            DataValue::U16(word) => word as u32,
            DataValue::U32(word) => word
        }
    }
}

impl fmt::Debug for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DataValue::U8(byte) => write!(f, "{:?}", byte as char),
            DataValue::U16(word) => write!(f, "0x{:04x}", word),
            DataValue::U32(word) => write!(f, "0x{:08x}", word),
        }
    }
}

/// Relation of timestamp to other trace events
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TimestampSync {
    /// Timestamp is synchronous to data packet
    Synchronous,
    
    /// The timestamp packet is delayed compared to data packet
    TimestampDelayed,

    /// The data packet is delayed compared to associated event
    DataDelayed,

    /// Both timestamp and data packets are delayed
    BothDelayed
}

/// Stores information about which event counters have overflowed/wrapped.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct EventCounterFlags {
    pub cpicnt: bool,
    pub exccnt: bool,
    pub sleepcnt: bool,
    pub lsucnt: bool,
    pub foldcnt: bool,
    pub postcnt: bool
}

/// Exception entry/exit event
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExceptionEvent {
    /// Exception handler started from beginning.
    Enter,
    
    /// Exception handler finished.
    Exit,
    
    /// Exception handler resumed execution after being pre-empted by
    /// higher priority exception.
    Resume
}

/// Extended information
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExtendedInformation {
    pub data: u32,
    pub bitcount: u8,
    pub source: u8
}