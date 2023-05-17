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

fn float_comma(input: Input) -> Result<f32> {
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
        |s| s.replace(',', ".").replace('_', "").as_str().parse::<f32>(),
    )(input)
}

fn float_dot(input: Input) -> Result<f32> {
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
        |s| s.replace('_', "").as_str().parse::<f32>(),
    )(input)
}

fn vec3(input: Input) -> Result<(f32, f32, f32)> {
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

fn quat(input: Input) -> Result<(f32, f32, f32, f32)> {
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

#[derive(Debug, Serialize)]
pub struct Device {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub rotation_euler: (f32, f32, f32),
}

fn device(input: Input) -> Result<Device> {
    map(tuple((vec3, multispace1, quat, multispace1, vec3)), |v| {
        Device {
            position: v.0,
            rotation: v.2,
            rotation_euler: v.4,
        }
    })(input)
}

#[derive(Debug, Serialize)]
pub struct LocalDevice {
    pub id: u32,
    pub device: Device,
}

fn local_device(input: Input) -> Result<LocalDevice> {
    map(
        tuple((tag("local"), multispace1, id, multispace1, device)),
        |v| LocalDevice {
            id: v.2,
            device: v.4,
        },
    )(input)
}

#[derive(Debug, Serialize)]
pub struct RemoteDevice {
    pub id: u32,
    pub interaction_profile: String,
    pub subaction_path: String,
    pub device: Device,
}

fn pseudopath(input: Input) -> Result<&str> {
    recognize(many1(one_of("abcdefghijklmnopqrstuvwxyz/_")))(input)
}

fn remote_device(input: Input) -> Result<RemoteDevice> {
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
            device,
        )),
        |v| RemoteDevice {
            id: v.2,
            interaction_profile: v.4.to_string(),
            subaction_path: v.6.to_string(),
            device: v.8,
        },
    )(input)
}

#[derive(Debug, Serialize)]
pub struct Line {
    pub time: f32,
    pub local: Vec<LocalDevice>,
    pub remote: Vec<RemoteDevice>,
}

impl Line {
    fn parse(input: Input) -> Result<Self> {
        map(
            tuple((
                float_comma,
                multispace1,
                separated_list0(multispace1, local_device),
                multispace1,
                separated_list0(multispace1, remote_device),
            )),
            |v| Self {
                time: v.0,
                local: v.2,
                remote: v.4,
            },
        )(input)
    }
}

pub fn file(input: Input) -> Result<Vec<Line>> {
    many0(map(tuple((Line::parse, multispace0)), |r| r.0))(input)
}
