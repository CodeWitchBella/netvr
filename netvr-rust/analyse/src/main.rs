mod parse;
mod plot;

use anyhow::Result;
use parse::{LogFile, Sample};
use serde_json::json;

fn main() -> Result<()> {
    // read file from argv
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename)?;

    // parse contents using nom
    let (_, file) = LogFile::parse(&contents)?;

    if false {
        std::fs::write(
            "output.json",
            serde_json::to_string(&serde_json::to_value(file.clone())?)?,
        )?;
    }

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

    // write to filename
    // replace ext from filename with .anal
    std::fs::write(
        std::path::Path::new(filename).with_extension("anal"),
        serde_json::to_string_pretty(&json!({
            "local_id": local_id,
            "local_characteristics": local_info.characteristics,
            "remote_id": remote_id,
            "remote_interaction_profile": remote_info.interaction_profile,
            "remote_subaction_path": remote_info.subaction_path,
            "local": local.iter().map(|s| s.position).collect::<Vec<_>>(),
            "remote": remote.iter().map(|s| s.position).collect::<Vec<_>>(),
        }))?,
    )?;

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
        local: local_distances,
        remote: remote_distances,
        out_file_name: std::path::Path::new(filename)
            .with_extension("png")
            .to_str()
            .unwrap()
            .to_string(),
    })?;

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
