extern crate babystats;
extern crate chrono;

use babystats::{BabyManagerData,Event};
use std::collections::BTreeMap;
use std::io;
use std::process;
use std::error::Error;
use chrono::offset::Local;

fn run() -> Result<(), Box<Error>> {
    let mut rdr = BabyManagerData::from_reader(io::stdin());
    let mut events_by_date: BTreeMap<chrono::Date<Local>, Vec<Event>> = BTreeMap::new();
    for event in &mut rdr.into_iter().map(|r| r.unwrap()) {
        events_by_date.entry(event.time().date()).or_insert(Vec::new()).push(event.clone());
    }
    let max_sleep_by_date: Vec<_> = events_by_date.into_iter().map(|(_, v)| {
        v.into_iter().filter_map(|r| {
            match r {
                Event::Sleep(sr) => Some(sr),
                _ => None,
            }
        }).fold(None, |acc, r| {
            match acc {
                None => Some(r),
                Some(a) => if a.duration > r.duration {
                    Some(a)
                } else {
                    Some(r)
                }
            }
        })
    }).filter_map(|v| v).collect();
    for sr in max_sleep_by_date {
        println!("{:?}: {}", sr.start.date(), duration_str(sr.duration))
    }
    Ok(())
}

fn duration_str(mut d: chrono::Duration) -> String {
    let hours = d.num_hours();
    d = d - chrono::Duration::hours(hours);
    let minutes = d.num_minutes();
    d = d - chrono::Duration::minutes(minutes);
    let seconds = d.num_seconds();
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
