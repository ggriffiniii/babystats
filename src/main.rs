extern crate csv;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate chrono;
extern crate regex;

use std::error::Error;
use std::io;
use std::process;
use chrono::offset::{Local,Utc};
use chrono::TimeZone;
use regex::Regex;

#[derive(Debug,Deserialize)]
struct RawRecord {
    #[serde(rename = "Type")]
	typ: String,
    #[serde(rename = "Start")]
	start: String,
    #[serde(rename = "End")]
	end: String,
    #[serde(rename = "Duration")]
	duration: String,
    #[serde(rename = "Extra")]
	extra: String,
    #[serde(rename = "Extra2")]
	extra2: String,
    #[serde(rename = "Note")]
	note: String,
}

impl RawRecord {
    fn into_record(self) -> Result<Record, Box<Error>> {
        match self.typ.as_str() {
            "Sleep" => Ok(Record::Sleep(self.to_sleep_record()?)),
            "Diaper" => Ok(Record::Diaper(self.to_diaper_record()?)),
            "Bottle feeding" => Ok(Record::Feeding(FeedingRecord::Bottle(self.to_bottle_record()?))),
            "Left breast" => Ok(Record::Feeding(FeedingRecord::LeftBreast(self.to_breast_record()?))),
            "Right breast" => Ok(Record::Feeding(FeedingRecord::RightBreast(self.to_breast_record()?))),
            "Pumping" => Ok(Record::Pumping(self.to_pumping_record()?)),
            "Vaccination" => Ok(Record::TummyTime(self.to_tummy_time_record()?)),
            "Measure" => Ok(Record::Measure(self.to_measure_record()?)),
            "Note" => Ok(Record::Note(self.to_note_record()?)),
            _ => Err(From::from(format!("unknown type: {}", self.typ))),
        }
    }

    fn to_sleep_record(&self) -> Result<SleepRecord, Box<Error>> {
        Ok(SleepRecord{
            start: datetime_from_str(&self.start)?,
            end:
                if self.end.len() == 0 {
                    None
                } else {
                    Some(datetime_from_str(&self.end)?)
                },
            duration: self.to_duration()?,
            note: self.note.clone(),
        })
    }

    fn to_diaper_record(&self) -> Result<DiaperRecord, Box<Error>> {
        Ok(DiaperRecord{
            time: datetime_from_str(&self.start)?,
            pee: self.extra.contains("Urine"),
            poo: self.extra.contains("Feces"),
            note: self.note.clone(),
        })
    }

    fn to_bottle_record(&self) -> Result<BottleRecord, Box<Error>> {
        Ok(BottleRecord{
            time: datetime_from_str(&self.start)?,
            milk: match self.extra2.as_str() {
                "Mom's milk" => Milk::BreastMilk,
                "Formula" => Milk::Formula,
                _ => Milk::Unknown,
            },
            ounces: {
                if self.extra.ends_with(" oz") {
                    self.extra[..self.extra.len()-3].parse::<f32>()?
                } else {
                    0.0
                }
            },
            note: self.note.clone(),
        })
    }

    fn to_breast_record(&self) -> Result<BreastRecord, Box<Error>> {
        Ok(BreastRecord{
            start: datetime_from_str(&self.start)?,
            end:
                if self.end.len() == 0 {
                    None
                } else {
                    Some(datetime_from_str(&self.end)?)
                },
            duration: self.to_duration()?,
            note: self.note.clone(),
        })
    }

    fn to_pumping_record(&self) -> Result<PumpingRecord, Box<Error>> {
        Ok(PumpingRecord{
            start: datetime_from_str(&self.start)?,
            ounces: {
                if self.extra.ends_with(" oz") {
                    self.extra[..self.extra.len()-3].parse::<f32>()?
                } else {
                    0.0
                }
            },
            note: self.note.clone(),
        })
    }

    fn to_tummy_time_record(&self) -> Result<TummyTimeRecord, Box<Error>> {
        Ok(TummyTimeRecord{
            start: datetime_from_str(&self.start)?,
            end:
                if self.end.len() == 0 {
                    None
                } else {
                    Some(datetime_from_str(&self.end)?)
                },
            duration: self.to_duration()?,
            note: self.note.clone(),
        })
    }

    fn to_measure_record(&self) -> Result<MeasureRecord, Box<Error>> {
        lazy_static! {
            static ref WEIGHT_RE: Regex = Regex::new(r"Weight: (\d+(?:\.\d+)?) lb").unwrap();
            static ref HEIGHT_RE: Regex = Regex::new(r"Height: (\d+(?:\.\d+)?) in").unwrap();
            static ref HEAD_CIRC_RE: Regex = Regex::new(r"Head circumference: (\d+(?:\.\d+)?) in").unwrap();
        }
        let weight = WEIGHT_RE.
            captures(self.extra.as_str()).
            and_then(|x| {
                x.get(1).unwrap().as_str().parse::<f32>().ok()
            });
        let height = HEIGHT_RE.
            captures(self.extra.as_str()).
            and_then(|x| {
                x.get(1).unwrap().as_str().parse::<f32>().ok()
            });
        let head_circ = HEAD_CIRC_RE.
            captures(self.extra.as_str()).
            and_then(|x| {
                x.get(1).unwrap().as_str().parse::<f32>().ok()
            });
        Ok(MeasureRecord{
            time: datetime_from_str(&self.start)?,
            weight: weight,
            height: height,
            head_circ: head_circ,
            note: self.note.clone(),
        })
    }

    fn to_note_record(&self) -> Result<NoteRecord, Box<Error>> {
        Ok(NoteRecord{
            time: datetime_from_str(&self.start)?,
            note: self.note.clone(),
        })
    }

    fn to_duration(&self) -> Result<chrono::Duration, Box<Error>> {
        let v: Vec<&str> = self.duration.split(':').collect();
        if v.len() != 2 {
            return Err(From::from(format!("Unable to parse duration {}; Expecting HH:MM format", self.duration)))
        }
        let hours : i64 = v[0].parse()?;
        let minutes : i64 = v[1].parse()?;
        Ok(chrono::Duration::minutes(hours * 60 + minutes))
    }
}

fn datetime_from_str<T: AsRef<str>>(s: T) -> Result<chrono::DateTime<Local>, Box<Error>> {
    Ok(Utc.datetime_from_str(s.as_ref(), "%0d/%0m/%Y %H:%M")?.with_timezone(&Local))
}

#[derive(Debug)]
enum Record {
    Sleep(SleepRecord),
    Diaper(DiaperRecord),
    Feeding(FeedingRecord),
    Pumping(PumpingRecord),
    TummyTime(TummyTimeRecord),
    Measure(MeasureRecord),
    Note(NoteRecord),
}

impl Record {
    fn time(&self) -> chrono::DateTime<Local> {
        match self {
            &Record::Sleep(ref r) => r.start,
            &Record::Diaper(ref r) => r.time,
            &Record::Feeding(ref r) => r.time(),
            &Record::Pumping(ref r) => r.start,
            &Record::TummyTime(ref r) => r.start,
            &Record::Measure(ref r) => r.time,
            &Record::Note(ref r) => r.time,
        }
    }
}

#[derive(Debug)]
struct SleepRecord {
    start: chrono::DateTime<Local>,
    end: Option<chrono::DateTime<Local>>,
    duration: chrono::Duration,
    note: String,
}

#[derive(Debug)]
struct DiaperRecord {
    time: chrono::DateTime<Local>,
    pee: bool,
    poo: bool,
    note: String,
}

#[derive(Debug)]
enum FeedingRecord {
    Bottle(BottleRecord),
    LeftBreast(BreastRecord),
    RightBreast(BreastRecord),
}

impl FeedingRecord {
    fn time(&self) -> chrono::DateTime<Local> {
        match self {
            &FeedingRecord::Bottle(ref r) => r.time,
            &FeedingRecord::LeftBreast(ref r) => r.start,
            &FeedingRecord::RightBreast(ref r) => r.start,
        }
    }
}

#[derive(Debug)]
enum Milk {
    BreastMilk,
    Formula,
    Unknown,
}

#[derive(Debug)]
struct BottleRecord {
    time: chrono::DateTime<Local>,
    milk: Milk,
    ounces: f32,
    note: String,
}

#[derive(Debug)]
struct BreastRecord {
    start: chrono::DateTime<Local>,
    end: Option<chrono::DateTime<Local>>,
    duration: chrono::Duration,
    note: String,
}

#[derive(Debug)]
struct PumpingRecord {
    start: chrono::DateTime<Local>,
    ounces: f32,
    note: String,
}

#[derive(Debug)]
struct TummyTimeRecord {
    start: chrono::DateTime<Local>,
    end: Option<chrono::DateTime<Local>>,
    duration: chrono::Duration,
    note: String,
}

#[derive(Debug)]
struct MeasureRecord {
    time: chrono::DateTime<Local>,
    weight: Option<f32>,
    height: Option<f32>,
    head_circ: Option<f32>,
    note: String,
}

#[derive(Debug)]
struct NoteRecord {
    time: chrono::DateTime<Local>,
    note: String,
}

fn run() -> Result<(), Box<Error>> {
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize::<RawRecord>() {
        let record = result?.into_record()?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    println!("Hello, world!");
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
