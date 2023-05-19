use std::cmp::{max_by, Ordering::Equal};

use anyhow::Result;
use plotters::prelude::*;

pub struct PlotInput {
    pub times: Vec<f64>,
    pub local: Vec<(f64, f64, f64)>,
    pub remote: Vec<(f64, f64, f64)>,
    pub out_file_name: String,
    pub recenter: bool,
}

fn f64_cmp(a: &f64, b: &f64) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(Equal)
}

fn range_on_axis(
    local: &[(f64, f64, f64)],
    remote: &[(f64, f64, f64)],
    map: fn(v: (f64, f64, f64)) -> f64,
) -> Option<std::ops::Range<f64>> {
    Some(
        local
            .iter()
            .chain(remote.iter())
            .map(|v| map(*v))
            .min_by(f64_cmp)?
            ..local
                .iter()
                .chain(remote.iter())
                .map(|v| map(*v))
                .max_by(f64_cmp)?,
    )
}

fn range_size(range: &std::ops::Range<f64>) -> f64 {
    (range.end - range.start).abs()
}

fn expand_range(range: &std::ops::Range<f64>, target_size: f64) -> std::ops::Range<f64> {
    let size = range_size(range);
    let diff = target_size - size;
    range.start - diff / 2.0..range.end + diff / 2.0
}

pub fn plot(input: PlotInput) -> Result<()> {
    let area = SVGBackend::new(&input.out_file_name, (1024, 760)).into_drawing_area();

    area.fill(&WHITE)?;

    let start = if input.recenter {
        input.local[0]
    } else {
        (0.0, 0.0, 0.0)
    };
    let local = input
        .local
        .iter()
        .map(|v| (v.0 - start.0, v.1 - start.1, v.2 - start.2))
        .collect::<Vec<_>>();
    let start = if input.recenter {
        input.remote[0]
    } else {
        (0.0, 0.0, 0.0)
    };
    let remote = input
        .remote
        .iter()
        .map(|v| (v.0 - start.0, v.1 - start.1, v.2 - start.2))
        .collect::<Vec<_>>();

    let x_axis = range_on_axis(&local, &remote, |v| v.0).unwrap();
    let y_axis = range_on_axis(&local, &remote, |v| v.1).unwrap();
    let z_axis = range_on_axis(&local, &remote, |v| v.2).unwrap();

    let max_range_size = max_by(
        max_by(range_size(&x_axis), range_size(&y_axis), f64_cmp),
        range_size(&z_axis),
        f64_cmp,
    );
    let x_axis = expand_range(&x_axis, max_range_size).step(0.01);
    let y_axis = expand_range(&y_axis, max_range_size).step(0.01);
    let z_axis = expand_range(&z_axis, max_range_size).step(0.01);

    let mut chart = ChartBuilder::on(&area)
        .caption(format!("3D Plot Test"), ("sans", 20))
        .build_cartesian_3d(x_axis.clone(), y_axis.clone(), z_axis.clone())?;

    chart.with_projection(|mut pb| {
        pb.yaw = 0.5;
        pb.scale = 0.9;
        pb.into_matrix()
    });

    chart
        .configure_axes()
        .light_grid_style(BLACK.mix(0.15))
        .max_light_lines(3)
        .draw()?;

    chart
        .draw_series(LineSeries::new(local, &BLUE))?
        .label("Local")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(LineSeries::new(remote, &GREEN))?
        .label("Remote")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart.configure_series_labels().border_style(BLACK).draw()?;

    // To avoid the IO failure being ignored silently, we manually call the present
    // function
    area.present().expect("Unable to write result to file");
    println!("Result has been saved to {}", input.out_file_name);
    Ok(())
}
