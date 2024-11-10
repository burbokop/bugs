mod env_presets;

use bugs_lib::{
    environment::SeededEnvironment,
    time_point::{StaticTimePoint, TimePoint as _},
    utils::{pretty_duration, Float},
};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Parser};
use memory_stats::memory_stats;
use rand_seeder::Seeder;
use serde::Serialize;
use std::{
    num::ParseIntError,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Args {
    New(NewCommand),
    Load(LoadCommand),
}

fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}

/// Generates simulation environment using provided seed
#[derive(Parser)]
struct NewCommand {
    #[arg(short, long)]
    seed: String,
    /// Timeout in seconds. Simulation will stop after reaching this time limit
    #[arg(short, long, value_parser = parse_duration)]
    timeout: Option<Duration>,
    /// If true, continuously checks memory in another thread and panics if it reaches maximum
    #[arg(long, action = ArgAction::Set, default_value = "true")]
    check_memory_usage: bool,
}

/// Loads simulation environment from json save file
#[derive(Parser)]
struct LoadCommand {
    file: PathBuf,
    /// Timeout in seconds. Simulation will stop after reaching this time limit
    #[arg(short, long, value_parser = parse_duration)]
    timeout: Option<Duration>,
    /// If true, continuously checks memory in another thread and panics if it reaches maximum
    #[arg(long, action = ArgAction::Set, default_value = "true")]
    check_memory_usage: bool,
}

fn save<T: Serialize>(environment: &SeededEnvironment<T>) {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let now: DateTime<Utc> = SystemTime::now().into();
    let save_path = exe_dir.join(format!("save_{}.json", now.format("%d.%m.%Y_%H:%M:%S")));
    println!("Saving into: {:?}", save_path);
    std::fs::write(
        &save_path,
        serde_json::to_string_pretty(&environment).unwrap(),
    )
    .unwrap();
}

fn main() {
    let args = Args::parse();
    let the_beginning_of_times = StaticTimePoint::default();

    let (mut environment, timeout, check_memory_usage) = match args {
        Args::New(command) => {
            println!("Run simulation with seed: {}", command.seed);
            (
                env_presets::less_food_further_from_center(
                    the_beginning_of_times.clone(),
                    Seeder::from(command.seed).make_seed(),
                ),
                command.timeout,
                command.check_memory_usage,
            )
        }
        Args::Load(command) => {
            println!("Run simulation from file: {:?}", command.file);
            (
                serde_json::from_str(&std::fs::read_to_string(command.file).unwrap()).unwrap(),
                command.timeout,
                command.check_memory_usage,
            )
        }
    };

    println!(
        "First bug genes: {:?}",
        environment.bugs().next().unwrap().chromosome().genes
    );

    if let Some(timeout) = timeout {
        println!("Timeout is set to: {}", pretty_duration(timeout));
    }

    println!("Check memory usage: {}", check_memory_usage);

    if check_memory_usage {
        std::thread::spawn(|| loop {
            if let Some(usage) = memory_stats() {
                if usage.physical_mem > 1024 * 1024 * 1024 {
                    panic!("Current memory usage exceeds limit: {:?}", usage);
                }
            } else {
                panic!("Couldn't get the current memory usage");
            }
            std::thread::sleep(Duration::from_secs(1));
        });
    }

    let sim_dt = Duration::from_millis(1000 / 30);
    let real_simulation_start_time = Instant::now();
    let mut last_cycle_instant = real_simulation_start_time.clone();
    let mut last_log_instant = real_simulation_start_time.clone();
    let mut last_save_instant = real_simulation_start_time.clone();
    while environment.bugs_count() > 0 {
        environment.proceed(sim_dt);
        let now = Instant::now();
        let real_dt = now - last_cycle_instant;
        last_cycle_instant = now;
        let time_speed = sim_dt.div_duration_f64(real_dt);

        if now - last_log_instant > Duration::from_secs(5) {
            println!(
                "Iteration {}, time: {}, population: {}, food: {}, time_speed: {:.2}, performance: {:.2}",
                environment.iteration(),
                pretty_duration(environment.now().duration_since(&the_beginning_of_times)),
                environment.bugs_count(),
                environment.food_count(),
                time_speed,
                environment.bugs_count() as Float * time_speed
            );
            last_log_instant = now
        }

        if now - last_save_instant > Duration::from_secs(60 * 5) {
            save(&environment);
            last_save_instant = now
        }

        if let Some(timeout) = timeout {
            if now - real_simulation_start_time > timeout {
                save(&environment);
                break;
            }
        }
    }
}
