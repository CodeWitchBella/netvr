use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1, one_of},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, many1, separated_list0},
    sequence::{terminated, tuple},
};
use serde::Serialize;

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

fn decimal(input: Input) -> Result<&str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

fn id(input: Input) -> Result<u32> {
    map_res(decimal, |r| r.parse::<u32>())(input)
}

fn float_comma(input: Input) -> Result<f64> {
    map_res(
        recognize(tuple((
            opt(one_of("+-")),
            alt((
                // Case one: .42
                recognize(tuple((char(','), decimal))),
                // Case two: 42. and 42.42
                recognize(tuple((decimal, char(','), opt(decimal)))),
            )),
        ))),
        |s| s.replace(',', ".").replace('_', "").as_str().parse::<f64>(),
    )(input)
}

fn float_dot(input: Input) -> Result<f64> {
    map_res(
        recognize(tuple((
            opt(one_of("+-")),
            alt((
                // Case one: .42
                recognize(tuple((char('.'), decimal))),
                // Case two: 42. and 42.42
                recognize(tuple((decimal, char('.'), opt(decimal)))),
            )),
        ))),
        |s| s.replace('_', "").as_str().parse::<f64>(),
    )(input)
}

fn vec3(input: Input) -> Result<(f64, f64, f64)> {
    map(
        tuple((
            char('('),
            float_dot,
            char(','),
            multispace0,
            float_dot,
            char(','),
            multispace0,
            float_dot,
            char(')'),
        )),
        |t| (t.1, t.4, t.7),
    )(input)
}

fn quat(input: Input) -> Result<(f64, f64, f64, f64)> {
    map(
        tuple((
            char('('),
            float_dot,
            char(','),
            multispace0,
            float_dot,
            char(','),
            multispace0,
            float_dot,
            char(','),
            multispace0,
            float_dot,
            char(')'),
        )),
        |t| (t.1, t.4, t.7, t.10),
    )(input)
}

#[derive(Debug, Serialize, Clone)]
pub struct Sample {
    pub position: (f64, f64, f64),
    pub rotation: (f64, f64, f64, f64),
    pub rotation_euler: (f64, f64, f64),
}

impl Sample {
    fn parse(input: Input) -> Result<Self> {
        map(tuple((vec3, multispace1, quat, multispace1, vec3)), |v| {
            Sample {
                position: v.0,
                rotation: v.2,
                rotation_euler: v.4,
            }
        })(input)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LocalSample {
    pub id: u32,
    pub characteristics: String,
    pub sample: Sample,
}

fn characteristics(input: Input) -> Result<&str> {
    recognize(many1(one_of(
        "abcdefghijklmnopqrstuvwxyz,ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    )))(input)
}

impl LocalSample {
    fn parse(input: Input) -> Result<Self> {
        map(
            tuple((
                tag("local"),
                multispace1,
                id,
                multispace1,
                characteristics,
                multispace1,
                Sample::parse,
            )),
            |v| LocalSample {
                id: v.2,
                characteristics: v.4.to_string(),
                sample: v.6,
            },
        )(input)
    }
}

impl From<LocalSample> for Sample {
    fn from(val: LocalSample) -> Self {
        val.sample
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RemoteSample {
    pub id: u32,
    pub interaction_profile: String,
    pub subaction_path: String,
    pub sample: Sample,
}

fn pseudopath(input: Input) -> Result<&str> {
    recognize(many1(one_of("abcdefghijklmnopqrstuvwxyz/_")))(input)
}

impl RemoteSample {
    fn parse(input: Input) -> Result<Self> {
        map(
            tuple((
                tag("remote"),
                multispace1,
                id,
                multispace1,
                pseudopath,
                multispace1,
                pseudopath,
                multispace1,
                Sample::parse,
            )),
            |v| Self {
                id: v.2,
                interaction_profile: v.4.to_string(),
                subaction_path: v.6.to_string(),
                sample: v.8,
            },
        )(input)
    }
}

impl From<RemoteSample> for Sample {
    fn from(val: RemoteSample) -> Self {
        val.sample
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Line {
    pub time: f64,
    pub local: HashMap<u32, LocalSample>,
    pub remote: HashMap<u32, RemoteSample>,
}

impl Line {
    fn parse(input: Input) -> Result<Self> {
        map(
            tuple((
                float_comma,
                multispace1,
                separated_list0(multispace1, LocalSample::parse),
                multispace1,
                separated_list0(multispace1, RemoteSample::parse),
            )),
            |v| Self {
                time: v.0,
                local: HashMap::from_iter(v.2.into_iter().map(|d| (d.id, d))),
                remote: HashMap::from_iter(v.4.into_iter().map(|d| (d.id, d))),
            },
        )(input)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LogFile {
    pub lines: Vec<Line>,
}

impl LogFile {
    pub fn parse(input: Input) -> Result<Self> {
        map(
            many0(map(tuple((Line::parse, multispace0)), |r| r.0)),
            |lines| Self { lines },
        )(input)
    }

    pub fn local_ids(&self) -> Vec<u32> {
        self.lines
            .iter()
            .flat_map(|l| l.local.keys().copied())
            .collect()
    }

    pub fn local(&self, id: u32) -> Vec<Sample> {
        self.lines
            .iter()
            .filter_map(|l| l.local.get(&id))
            .map(|s| s.sample.clone())
            .collect()
    }

    pub fn remote_ids(&self) -> Vec<u32> {
        self.lines
            .iter()
            .flat_map(|l| l.remote.keys().copied())
            .collect()
    }

    pub fn remote(&self, id: u32) -> Vec<Sample> {
        self.lines
            .iter()
            .filter_map(|l| l.remote.get(&id.clone()))
            .map(|s| s.sample.clone())
            .collect()
    }
}
