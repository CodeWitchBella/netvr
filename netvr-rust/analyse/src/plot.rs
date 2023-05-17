use std::cmp::{max_by, min_by, Ordering::Equal};

use anyhow::Result;
use plotters::prelude::*;

pub struct PlotInput {
    pub times: Vec<f64>,
    pub local: Vec<f64>,
    pub remote: Vec<f64>,
    pub out_file_name: String,
}

fn f64_cmp(a: &&f64, b: &&f64) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(Equal)
}

pub fn plot(input: PlotInput) -> Result<()> {
    let root = BitMapBackend::new(&input.out_file_name, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let min = *min_by(
        input.local.iter().min_by(f64_cmp).unwrap(),
        input.remote.iter().min_by(f64_cmp).unwrap(),
        f64_cmp,
    );

    let max = *max_by(
        input.local.iter().max_by(f64_cmp).unwrap(),
        input.remote.iter().max_by(f64_cmp).unwrap(),
        f64_cmp,
    );

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Distance from start over time", ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(input.times[0]..*input.times.last().unwrap(), min..max)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .max_light_lines(4)
        //.y_desc("")
        .draw()?;

    chart.draw_series(LineSeries::new(
        input
            .local
            .iter()
            .enumerate()
            .map(|(i, v)| (input.times[i], *v)),
        &BLUE,
    ))?;
    chart.draw_series(LineSeries::new(
        input
            .remote
            .iter()
            .enumerate()
            .map(|(i, v)| (input.times[i], *v)),
        &GREEN,
    ))?;

    println!("Result has been saved to {}", input.out_file_name);
    Ok(())
}
