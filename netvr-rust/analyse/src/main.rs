mod parse;
mod plot;
mod plot_positions;

use anyhow::Result;
use parse::{LogFile, Sample};

use crate::parse::Line;

fn main() -> Result<()> {
    // read file from argv
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename)?;

    // parse contents using nom
    let (rest, file) = LogFile::parse(&contents)?;
    if !rest.is_empty() {
        println!("Warning: unparsed data: {:?}", rest);
    }
    let start_time = file.lines[0].time;
    let file = LogFile {
        lines: file
            .lines
            .into_iter()
            .map(|l| Line {
                time: l.time - start_time,
                ..l
            })
            .collect(),
    };

    if false {
        std::fs::write(
            "output.json",
            serde_json::to_string(&serde_json::to_value(file.clone())?)?,
        )?;
    }
    println!("Lines: {}", file.lines.len());

    let (local_id, local) = file
        .local_ids()
        .iter()
        .map(|id| (*id, file.local(*id)))
        .reduce(|a, b| {
            let da = distance_traveled(&a.1);
            let db = distance_traveled(&b.1);
            if da > db {
                a
            } else {
                b
            }
        })
        .expect("At least one local device must be present");
    let (remote_id, remote) = file
        .remote_ids()
        .iter()
        .map(|id| (*id, file.remote(*id)))
        .reduce(|a, b| {
            let da = distance_traveled(&a.1);
            let db = distance_traveled(&b.1);
            if da > db {
                a
            } else {
                b
            }
        })
        .expect("At least one remote device must be present");

    let remote_info = file.lines[0].remote.get(&remote_id).unwrap();
    println!(
        "Remote device: {:?} {:?} {:?}",
        remote_info.id, remote_info.interaction_profile, remote_info.subaction_path
    );
    println!("  traveled: {:?}", distance_traveled(&remote));
    let local_info = file.lines[0].local.get(&local_id).unwrap();
    println!(
        "Local device: {:?} {:?}",
        local_id, local_info.characteristics
    );
    println!("  traveled: {:?}", distance_traveled(&local));

    let local_distances = map_to_distance_from_start(&local);
    let remote_distances = map_to_distance_from_start(&remote);

    std::fs::write(
        std::path::Path::new(filename).with_extension("csv"),
        local_distances
            .iter()
            .zip(remote_distances.iter())
            .zip(file.lines.iter().map(|l| l.time))
            .fold("time;local;remote\n".to_string(), |acc, ((l, r), time)| {
                format!("{acc}{time};{l};{r}\n").replace('.', ",")
            }),
    )?;

    plot::plot(plot::PlotInput {
        times: file.lines.iter().map(|l| l.time).collect(),
        local: local_distances.clone(),
        remote: remote_distances.clone(),
        out_file_name: std::path::Path::new(filename)
            .with_extension("svg")
            .to_str()
            .unwrap()
            .to_string(),
    })?;

    plot_positions::plot(plot_positions::PlotInput {
        times: file.lines.iter().map(|l| l.time).collect(),
        local: local.iter().map(|l| l.position).collect(),
        remote: remote.iter().map(|l| l.position).collect(),
        out_file_name: std::path::Path::new(filename)
            .with_extension("3d.svg")
            .to_str()
            .unwrap()
            .to_string(),
        recenter: false,
    })?;

    std::fs::write(
        std::path::Path::new(filename).with_extension("dat"),
        local_distances
            .iter()
            .zip(remote_distances.iter())
            .fold("local\tremote\n".to_string(), |acc, (l, r)| {
                format!("{acc}{l}\t{r}\n")
            }),
    )?;
    std::fs::write(
        std::path::Path::new(filename).with_extension("dat.csv"),
        local_distances
            .iter()
            .zip(remote_distances.iter())
            .fold("local;remote\n".to_string(), |acc, (l, r)| {
                format!("{acc}{l};{r}\n")
            }),
    )?;

    Ok(())
}

fn map_to_distance_from_start(samples: &[Sample]) -> Vec<f64> {
    let start = samples[0].clone();
    samples.iter().map(|s| distance(s, &start)).collect()
}

fn distance_traveled(samples: &[Sample]) -> f64 {
    samples
        .iter()
        .zip(samples.iter().skip(1))
        .map(|(a, b)| distance(a, b))
        .sum()
}

fn distance(a: &Sample, b: &Sample) -> f64 {
    let dx = b.position.0 - a.position.0;
    let dy = b.position.1 - a.position.1;
    let dz = b.position.2 - a.position.2;
    (dx * dx + dy * dy + dz * dz).sqrt()
}
