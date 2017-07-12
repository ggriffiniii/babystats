extern crate babystats;
extern crate chrono;

use chrono::Timelike;
use babystats::{BabyManagerData,Event,SleepEvent};
use std::collections::BTreeMap;
use std::io;
use std::process;
use std::error::Error;

fn run() -> Result<(), Box<Error>> {
    let mut rdr = BabyManagerData::from_reader(io::stdin());
    let sleep_events = &mut rdr.into_iter().filter_map(|e| {
        match e {
            Ok(Event::Sleep(se @ SleepEvent{end: Some(_), ..})) => Some(se),
            _ => None,
        }
    });
    let mut sleeps_by_date: BTreeMap<_, _> = BTreeMap::new();
    for event in sleep_events {
        sleeps_by_date.entry(event.end.unwrap().date()).or_insert(Vec::new()).push(event.clone());
    }
    for (_, v) in &mut sleeps_by_date {
        v.sort_by_key(|e| e.start);
    }
    let wakeups_by_date: BTreeMap<_, _> = sleeps_by_date.into_iter().map(|(k, v)| {
        let mut prev: Option<SleepEvent> = None;
        let mut wakeups: i32 = 0;
        for e in v {
            println!("{:?}, {:?}", e.end.unwrap(), e.end.unwrap().hour());
            if e.end.unwrap().hour() > 10 {
                println!("after 10am");
                break;
            }
            if let Some(pe) = prev {
                if pe.end.unwrap().signed_duration_since(e.start) > chrono::Duration::minutes(90) {
                    println!("90 minutes elapsed");
                    break;
                }
            }
            println!("waking up");
            wakeups += 1;
            prev = Some(e);
        }
        (k, wakeups)
    }).collect();
    for (date, wakeups) in wakeups_by_date {
        println!("{}: {}", date, wakeups)
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
