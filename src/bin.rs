extern crate arm_coresight_decoder;

use arm_coresight_decoder::itm::parser;
use arm_coresight_decoder::itm::heuristics;
//use arm_coresight_decoder::tpiu::parser;
use std::io::{Read,BufRead,Write};

pub fn main() {
    let mut input = std::io::BufReader::new(std::io::stdin());
    let mut output = std::io::BufWriter::new(std::io::stdout());
    // let mut packetcount: u32 = 0;

    //let mut parser = parser::Parser::new(Box::new(input));

//    for packet in parser.by_ref() {
  //      writeln!(&mut output, "0x{:08x}: {:?}", parser.position(), packet).unwrap();
    //}

    //if let Some(ref e) = parser.error() {
    //    output.flush().ok();
    //    if e.kind() != std::io::ErrorKind::UnexpectedEof
    //    {
    //        eprintln!("Error: {}", e);
    //    }
   // }

     let start = heuristics::find_starting_point(input.fill_buf().unwrap());
     println!("Start {}", start);
     let mut input = ReadPos::new(input);
     input.read(&mut vec![0; start]).ok();

     loop {
         let packet = parser::parse_one(&mut input);
         match packet {
             Ok(packet) => {
                 packetcount += 1;
                 writeln!(&mut output, "0x{:08x}: {:?}", input.position(), packet).unwrap();
             },
             Err(e) => {
                 output.flush().ok();
                 if e.kind() != std::io::ErrorKind::UnexpectedEof
                 {
                     eprintln!("Error: {}", e);
                 }
                 break;
             }
         }
    }

    println!("Total packets: {}", packetcount);
}
