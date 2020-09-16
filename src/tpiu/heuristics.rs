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
pub fn likelihood(packet: TPIUPacket) -> f32 {
    match packet {
        // Frame sync will always be properly synchronized afterwards.
        TPIUPacket::FrameSynchronization => 1.0,

        // Halfword sync is not as useful and quite rare.
        TPIUPacket::HalfwordSynchronization => 0.5,

        // For data packets, typically only lowest stream ID values are
        // used, and most contained packets are atleast 3 bytes.
        TPIUPacket::Data(id, payload) => {
            let id_prob = (if id.0 <= 5 {1.0} else {0.8};
            let data_prob = (if payload.length() >= 3 {1.0} else {0.8});
            id_prob * data_prob
        },

        // Trigger packet can have multiple payload bytes if multiple
        // triggers occur simultaneously, but usually it has just one.    
        TPIUPacket::Trigger(payload) => {
            if payload.length() == 1 {1.0} else {0.8};
        },

        // Null packets should have only 0 bytes as payload    
        TPIUPacket::Null(payload) => {
            if payload.all_equal(0) {1.0} else {0.5};
        },

        // Invalid/error types should not occur at all.
        TPIUPacket::Reserved(_) => 0.2,
        TPIUPacket::Invalid(_) => 0.0
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