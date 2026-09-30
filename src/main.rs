use std::{
    error::Error,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};

/// A lightweight CLI tool for benchmarking shell commands.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Command to benchmark.
    #[arg(short, long)]
    command: String,

    /// Number of benchmark runs.
    #[arg(
        short,
        long,
        default_value_t = 20,
        value_parser = clap::value_parser!(u32).range(1..)
    )]
    times: u32,

    /// Output time unit.
    #[arg(short, long, value_enum, default_value_t = TimeUnit::Ms)]
    output: TimeUnit,

    /// Display the command's stdout and stderr while benchmarking.
    #[arg(short, long)]
    display: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TimeUnit {
    /// Seconds
    S,

    /// Milliseconds
    Ms,

    /// Microseconds
    Us,
}

impl TimeUnit {
    fn convert(self, duration: Duration) -> f64 {
        match self {
            Self::S => duration.as_secs_f64(),
            Self::Ms => duration.as_secs_f64() * 1_000.0,
            Self::Us => duration.as_secs_f64() * 1_000_000.0,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::S => "s",
            Self::Ms => "ms",
            Self::Us => "us",
        }
    }
}

/// Run a shell command once and return its execution time.
fn measure(command: &str, display: bool) -> Result<Duration, Box<dyn Error>> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if !display {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let start = Instant::now();
    let status = cmd.status()?;
    let elapsed = start.elapsed();

    if !status.success() {
        return Err(format!("command exited with status: {status}").into());
    }

    Ok(elapsed)
}

fn median(values: &[f64]) -> f64 {
    let mid = values.len() / 2;

    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut results = Vec::with_capacity(args.times as usize);

    for _ in 0..args.times {
        let duration = measure(&args.command, args.display)?;
        results.push(args.output.convert(duration));
    }

    results.sort_by(f64::total_cmp);

    let min = results[0];
    let max = results[results.len() - 1];
    let median = median(&results);
    let mean = results.iter().sum::<f64>() / results.len() as f64;

    let unit = args.output.name();

    println!(
        "Runs:   {}\n\
         Min:    {:.3} {}\n\
         Max:    {:.3} {}\n\
         Median: {:.3} {}\n\
         Mean:   {:.3} {}",
        args.times,
        min,
        unit,
        max,
        unit,
        median,
        unit,
        mean,
        unit,
    );

    Ok(())
}
