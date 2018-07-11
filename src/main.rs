extern crate gnuplot;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use gnuplot::{AxesCommon, Caption, Color, Coordinate::*, Figure, LabelOption::*};

use std::{env, error::Error, fs, path::Path, process};

struct FramesInfo {
    caption: String,
    label: String,
    x: Vec<usize>,
    y: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
struct FrameMetrics {
    vmaf: f64,
    psnr: f64,
    ms_ssim: f64,
}

#[derive(Serialize, Deserialize)]
struct FrameInfo {
    #[serde(rename = "frameNum")]
    frame_idx: usize,
    metrics: FrameMetrics,
}

#[derive(Serialize, Deserialize)]
struct Frames {
    frames: Vec<FrameInfo>,
}

macro_rules! frames_info {
    ($lines:ident, $name:expr, $frames:ident, $metric:ident) => {
        let metric_name = stringify!($metric).to_uppercase();

        let y: Vec<_> = if metric_name.ends_with("SSIM") {
            $frames
                .iter()
                .map(|f| 1.0 / (1.0 - f.metrics.$metric))
                .collect()
        } else {
            $frames.iter().map(|f| f.metrics.$metric).collect()
        };

        let mut vals = y.clone();
        vals.sort_by(|&v1, &v2| ((v1 * 10000.0) as isize).cmp(&((v2 * 10000.0) as isize)));

        let avg: f64 = vals.iter().sum::<f64>() / (vals.len() as f64);
        let var: f64 = vals.iter().map(|v| (v - avg).powf(2.0)).sum::<f64>()
            / (vals.len() - 1) as f64;

        let label = format!(
            "{:2.3}  {:2.3}  {:2.3}  {:2.3}",
            vals[0],
            vals.last().unwrap(),
            avg,
            var
        );

        let frames_info = FramesInfo {
            caption: $name.to_string(),
            label,
            x: $frames.iter().map(|f| f.frame_idx).collect(),
            y,
        };

        $lines.push(frames_info);
    };
}

macro_rules! gen_figure {
    ($lines:ident, $metric:ident) => {
        let orig_metric_name = stringify!($metric).to_uppercase();
        let mut metric_name = orig_metric_name.clone();

        if metric_name.ends_with("SSIM") {
            metric_name = "1/(1-".to_owned() + &metric_name + ")";
        }

        let mut out_file = orig_metric_name;
        let mut fg = Figure::new();
        {
            let mut fg_2d = fg
                .axes2d()
                .set_x_label("Frames", &[])
                .set_y_label(&metric_name.replace('_', "\\\\\\_"), &[]);

            let colors = [
                "#990000", "#009900", "#000099", "#999900", "#990099", "#009999",
            ];
            let mut color_idx = 0;
            let mut offset = 0.22;

            let status_line = format!("{:^6}  {:^6}  {:^6}  {:^6}", "min", "max", "avg", "var");
            fg_2d = { fg_2d }.label(
                &status_line,
                Graph(0.02),
                Graph(offset),
                &[Font("monospace", 0.0)],
            );

            for line in &$lines {
                offset -= 0.05;
                fg_2d = { fg_2d }.label(
                    &line.label,
                    Graph(0.02),
                    Graph(offset),
                    &[TextColor(colors[color_idx]), Font("monospace", 0.0)],
                );
                fg_2d = { fg_2d }.lines(
                    &line.x,
                    &line.y,
                    &[
                        Caption(&line.caption.replace('_', "\\\\\\_")),
                        Color(colors[color_idx]),
                    ],
                );
                color_idx += 1;
                out_file += "-";
                out_file += &line.caption;
            }
        }

        out_file += ".png";
        println!("Writing \"{}\"", out_file);
        fg.set_terminal("pngcairo size 800, 600", &out_file)
            .show();
    };
}

fn main() -> Result<(), Box<Error>> {
    let prog_path = env::args().next().ok_or("No $0 !!!")?;

    if env::args().count() == 1 {
        eprintln!("Usage: {} vmaf_json_file1 [vmaf_json_file2 ...]", prog_path);
        process::exit(1);
    }

    let mut args = env::args();
    args.next(); // skip $0

    let mut vmaf_lines: Vec<FramesInfo> = Vec::with_capacity(4);
    let mut ms_ssim_lines: Vec<FramesInfo> = Vec::with_capacity(4);
    let mut psnr_lines: Vec<FramesInfo> = Vec::with_capacity(4);

    for arg in args {
        let j_arg_filename = arg;
        let j_path = Path::new(&j_arg_filename);

        if !j_path.exists() || j_path.is_dir() {
            eprintln!("{} is a dir or does not exist", j_arg_filename);
            process::exit(1);
        }

        let j_filename = j_path
            .file_name()
            .ok_or("Impossible")?
            .to_str()
            .ok_or("Impossible")?;

        let j_bytes = fs::read(j_path)?;
        let frames: Frames = serde_json::from_slice(&*j_bytes)?;
        let frames = frames.frames;

        frames_info!(vmaf_lines, j_filename, frames, vmaf);
        frames_info!(ms_ssim_lines, j_filename, frames, ms_ssim);
        frames_info!(psnr_lines, j_filename, frames, psnr);
    }

    gen_figure!(vmaf_lines, vmaf);
    gen_figure!(ms_ssim_lines, ms_ssim);
    gen_figure!(psnr_lines, psnr);

    Ok(())
}
