extern crate babystats;
extern crate chrono;

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
    let max_sleep_by_date: Vec<_> = sleeps_by_date.into_iter().map(|(_, v)| {
        v.into_iter().fold(None, |acc, r| {
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
    for sr in max_sleep_by_date.windows(5) {
        let (count, sum) = sr.iter().fold((0,0), |(c,s), sr| {
            (c + 1, s + sr.duration.num_milliseconds())
        });
        let mean = (sum as f64 / count as f64) as i64;
        let date = sr.iter().last().unwrap().end.unwrap().date();
        println!("{}: {}", date, duration_str(chrono::Duration::milliseconds(mean)))
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
