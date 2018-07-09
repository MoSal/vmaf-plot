extern crate serde_json;
extern crate textplots;

#[macro_use]
extern crate serde_derive;

use textplots::{Chart, Plot, Shape};

use std::{env, error::Error, fs, iter::FromIterator, path::Path, process};

macro_rules! gen_stats {
    ($frames:ident, $($metric:ident),+) => (
    $(
    let mut metric_name = stringify!($metric).to_uppercase();

    let points = if metric_name.ends_with("SSIM") {
        metric_name = "1/(1-".to_owned() + &metric_name + ")";
        let points_iter = $frames.iter().map(|f| (f.frame_idx, 1.0/(1.0-f.metrics.$metric) as f32));
        Vec::from_iter(points_iter)
    } else {
        let points_iter = $frames.iter().map(|f| (f.frame_idx, f.metrics.$metric as f32));
        Vec::from_iter(points_iter)
    };

    let mut vals : Vec<_> = points.iter().map(|p| p.1).collect();
    vals.sort_by(|&v1, &v2| ((v1 * 10000.0) as isize).cmp(&((v2 * 10000.0) as isize)));

    let avg: f32 = vals.iter().sum::<f32>() / (vals.len() as f32);
    let mid = vals[vals.len()/2];
    let div: f32 = (vals.iter().map(|v| (v-avg).powf(2.0)).sum::<f32>() / (vals.len() - 1) as f32).powf(0.5);

    println!("{:13} | (avg: {:3.3}, mid: {:3.3}, div: {:3.3}, min: {:3.3}, max: {:3.3})",
             metric_name, avg, mid, div, vals[0], vals.last().unwrap());
    Chart::new(80, 40, 0.0, points.len() as f32)
        .lineplot( Shape::Lines(&points) )
        .display();
    )+

    )
}

#[derive(Serialize, Deserialize)]
struct FrameMetrics {
    vmaf: f64,
    psnr: f64,
    ms_ssim: f64,
}

#[derive(Serialize, Deserialize)]
struct FrameInfo {
    #[serde(rename="frameNum")]
    frame_idx: f32,
    metrics: FrameMetrics,
}

#[derive(Serialize, Deserialize)]
struct Frames {
    frames: Vec<FrameInfo>,
}

fn main() -> Result<(), Box<Error>> {
    // Check no. of args
    if env::args().len() != 2 {
        eprintln!("Usage: {} json_file", env::args().nth(0).unwrap());
        process::exit(1);
    }

    let j_filename = env::args().nth(1).unwrap();
    let j_path = Path::new(&j_filename);

    if !j_path.exists() || j_path.is_dir() {
        eprintln!("{} is a dir or does not exist", j_filename);
        process::exit(1);
    }

    let j_bytes = fs::read(j_path)?;
    let frames: Frames = serde_json::from_slice(&*j_bytes)?;
    let frames = frames.frames;

    gen_stats!(frames, vmaf, ms_ssim, psnr);

    Ok(())
}
