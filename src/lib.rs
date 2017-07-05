extern crate csv;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate chrono;
extern crate regex;

use chrono::TimeZone;
use std::error::Error;
use std::io;
use chrono::offset::{Local,Utc};
use regex::Regex;

#[derive(Debug,Deserialize)]
struct RawEvent {
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

impl RawEvent {
    fn into_event(self) -> Result<Event, Box<Error>> {
        match self.typ.as_str() {
            "Sleep" => Ok(Event::Sleep(self.to_sleep_event()?)),
            "Diaper" => Ok(Event::Diaper(self.to_diaper_event()?)),
            "Bottle feeding" => Ok(Event::Feeding(FeedingEvent::Bottle(self.to_bottle_event()?))),
            "Left breast" => Ok(Event::Feeding(FeedingEvent::LeftBreast(self.to_breast_event()?))),
            "Right breast" => Ok(Event::Feeding(FeedingEvent::RightBreast(self.to_breast_event()?))),
            "Pumping" => Ok(Event::Pumping(self.to_pumping_event()?)),
            "Vaccination" => Ok(Event::TummyTime(self.to_tummy_time_event()?)),
            "Measure" => Ok(Event::Measure(self.to_measure_event()?)),
            "Note" => Ok(Event::Note(self.to_note_event()?)),
            _ => Err(From::from(format!("unknown type: {}", self.typ))),
        }
    }

    fn to_sleep_event(&self) -> Result<SleepEvent, Box<Error>> {
        Ok(SleepEvent{
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

    fn to_diaper_event(&self) -> Result<DiaperEvent, Box<Error>> {
        Ok(DiaperEvent{
            time: datetime_from_str(&self.start)?,
            pee: self.extra.contains("Urine"),
            poo: self.extra.contains("Feces"),
            note: self.note.clone(),
        })
    }

    fn to_bottle_event(&self) -> Result<BottleEvent, Box<Error>> {
        Ok(BottleEvent{
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

    fn to_breast_event(&self) -> Result<BreastEvent, Box<Error>> {
        Ok(BreastEvent{
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

    fn to_pumping_event(&self) -> Result<PumpingEvent, Box<Error>> {
        Ok(PumpingEvent{
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

    fn to_tummy_time_event(&self) -> Result<TummyTimeEvent, Box<Error>> {
        Ok(TummyTimeEvent{
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

    fn to_measure_event(&self) -> Result<MeasureEvent, Box<Error>> {
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
        Ok(MeasureEvent{
            time: datetime_from_str(&self.start)?,
            weight: weight,
            height: height,
            head_circ: head_circ,
            note: self.note.clone(),
        })
    }

    fn to_note_event(&self) -> Result<NoteEvent, Box<Error>> {
        Ok(NoteEvent{
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

#[derive(Debug,Clone)]
pub enum Event {
    Sleep(SleepEvent),
    Diaper(DiaperEvent),
    Feeding(FeedingEvent),
    Pumping(PumpingEvent),
    TummyTime(TummyTimeEvent),
    Measure(MeasureEvent),
    Note(NoteEvent),
}

impl Event {
    pub fn time(&self) -> chrono::DateTime<Local> {
        match self {
            &Event::Sleep(ref r) => r.start,
            &Event::Diaper(ref r) => r.time,
            &Event::Feeding(ref r) => r.time(),
            &Event::Pumping(ref r) => r.start,
            &Event::TummyTime(ref r) => r.start,
            &Event::Measure(ref r) => r.time,
            &Event::Note(ref r) => r.time,
        }
    }
}

#[derive(Debug,Clone)]
pub struct SleepEvent {
    pub start: chrono::DateTime<Local>,
    pub end: Option<chrono::DateTime<Local>>,
    pub duration: chrono::Duration,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct DiaperEvent {
    pub time: chrono::DateTime<Local>,
    pub pee: bool,
    pub poo: bool,
    pub note: String,
}

#[derive(Debug,Clone)]
pub enum FeedingEvent {
    Bottle(BottleEvent),
    LeftBreast(BreastEvent),
    RightBreast(BreastEvent),
}

impl FeedingEvent {
    pub fn time(&self) -> chrono::DateTime<Local> {
        match self {
            &FeedingEvent::Bottle(ref r) => r.time,
            &FeedingEvent::LeftBreast(ref r) => r.start,
            &FeedingEvent::RightBreast(ref r) => r.start,
        }
    }
}

#[derive(Debug,Clone)]
pub enum Milk {
    BreastMilk,
    Formula,
    Unknown,
}

#[derive(Debug,Clone)]
pub struct BottleEvent {
    pub time: chrono::DateTime<Local>,
    pub milk: Milk,
    pub ounces: f32,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct BreastEvent {
    pub start: chrono::DateTime<Local>,
    pub end: Option<chrono::DateTime<Local>>,
    pub duration: chrono::Duration,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct PumpingEvent {
    pub start: chrono::DateTime<Local>,
    pub ounces: f32,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct TummyTimeEvent {
    pub start: chrono::DateTime<Local>,
    pub end: Option<chrono::DateTime<Local>>,
    pub duration: chrono::Duration,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct MeasureEvent {
    pub time: chrono::DateTime<Local>,
    pub weight: Option<f32>,
    pub height: Option<f32>,
    pub head_circ: Option<f32>,
    pub note: String,
}

#[derive(Debug,Clone)]
pub struct NoteEvent {
    pub time: chrono::DateTime<Local>,
    pub note: String,
}

pub struct BabyManagerData<R> {
    rdr: csv::Reader<R>
}

impl<R: io::Read> BabyManagerData<R> {
    pub fn from_reader(rdr: R) -> BabyManagerData<R> {
        BabyManagerData{
            rdr: csv::Reader::from_reader(rdr),
        }
    }
}

impl<'a, R : io::Read> IntoIterator for &'a mut BabyManagerData<R> {
    type Item = Result<Event, Box<Error>>;
    type IntoIter = Iter<'a, R>;
    fn into_iter(self) -> Iter<'a, R> {
        Iter{iter: self.rdr.deserialize::<RawEvent>()}
    }
}

pub struct Iter<'a, R: 'a> {
    iter: csv::DeserializeRecordsIter<'a, R, RawEvent>
}

impl<'a, R : io::Read> Iterator for Iter<'a, R> {
    type Item = Result<Event, Box<Error>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Err(e)) => Some(Err(Box::new(e))),
            Some(Ok(raw_event)) => Some(raw_event.into_event()),
        }
    }
}