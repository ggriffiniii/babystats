extern crate babystats;
extern crate chrono;

use babystats::{BabyManagerData,Event};
use std::io;
use std::process;
use std::error::Error;

fn run() -> Result<(), Box<Error>> {
    let mut rdr = BabyManagerData::from_reader(io::stdin());
    let pump_events = &mut rdr.into_iter().filter_map(|e| {
        match e {
            Ok(Event::Pumping(pe)) => Some(pe),
            _ => None,
        }
    });
    for event in pump_events {
        //if let (Some(l), Some(r)) = (event.left_ml, event.right_ml) {
        //    println!("{}L + {}R == {} == {}", l, r, l+r, event.ml);
        //}
        //println!("{:?}, {:?}, {:?}, {:?}", event.start, event.note, event.left_ml, event.right_ml);
        if event.left_ml.is_none() || event.right_ml.is_none() {
        }
        println!("{:?}", event);
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
