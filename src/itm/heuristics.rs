//! Regains stream synchronization by heuristically estimating
//! packet probabilities.

use super::types::*;
use super::parser::Parser;
use std::io::Cursor;

/// Returns float in range 0.0 .. 1.0, where 0.0 is least likely
/// packet type. The value is an estimate for how likely this packet
/// is part of a properly synced datastream. Very rare packets
/// have low probability, while packets that don't easily occur
/// in missynced streams have high probability.
pub fn likelihood(packet: ITMPacket) -> f32 {
    match packet {
        // Synchronization packet will always be properly parsed,
        // even if the stream was missynced before it.
        ITMPacket::Synchronization => 1.0,

        // Overflow packet is single byte with value 0x70, so
        // doesn't easily get missynced.
        ITMPacket::Overflow => 0.9,

        // ITM software trace packets are common but get quite easily
        // missynced also.
        ITMPacket::Software(_,_) => 0.4,

        // Rare packets
        ITMPacket::Extension(_) => 0.2,
        ITMPacket::Reserved(_) => 0.1,
        ITMPacket::Invalid(_) => 0.0,

        // Default value
        _ => 0.5
    }
}

pub fn find_starting_point(block: &[u8]) -> usize {
    (0..4).rev().map(|i| {
        let iter = Parser::new(Cursor::new(&block[i..]));
        let mut score = 0.0;
        let mut count = 0;
        for packet in iter {
            score += likelihood(packet);
            count += 1;
        }
        score /= count as f32;
        println!("i {} score {}", i, score);
        (i,score)
    }).max_by(|&(_,a), &(_,b)| a.partial_cmp(&b).unwrap()).unwrap_or((0,0.0)).0
}

