mod data_structs;
mod solver;

use data_structs::{Block, ErrorResult, ProblemParams, Vec3};
use glob::{glob, GlobError};
use log::{debug, error, info, warn};
use simple_error::{bail, SimpleError};
use solver::get_pos_index;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use crate::solver::solve_problem;

fn main() {
    let log_level = log::LevelFilter::from_str(&std::env::var("RUST_LOG").unwrap_or("info".to_string())).unwrap_or(log::LevelFilter::Info);
    env_logger::builder().filter_level(log_level).format_target(true).format_timestamp(None).init();

    let args: Vec<String> = args().collect();
    let result: ErrorResult<()> = if let Some(path) = args.get(1) {
        let attemps = args.get(2).and_then(|x| str::parse::<usize>(x).ok().filter(|&n| n > 1));
        let do_threaded = args.get(3).and_then(|x| str::parse::<bool>(x).ok()).unwrap_or(false);
        run(path, attemps, do_threaded)
    } else {
        
    let now = Instant::now();
        // let do_threaded = args.get(1).and_then(|x| str::parse::<bool>(x).ok()).unwrap_or(false);
        let r = run_all(true);
        println!("{} ms", now.elapsed().as_millis());
        r
    };

    if let Err(e) = result {
        error!("{}", e);
    }
}

pub fn find_average_time(params: &mut ProblemParams, attempts: usize, do_threaded: bool) {
    let mut cumulative_micros: u128 = 0;

    for i in 0..attempts {
        let (solution, elapsed_micros) = solve_problem(params, do_threaded);
        cumulative_micros += elapsed_micros;
        info!("Attempt {}: Solving took {} ms!", i + 1, (elapsed_micros as f64) / 1000f64);
        if let None = solution {
            warn!("No solution!");
        }
    }
    info!("Solving took {} ms on average!", (cumulative_micros as f64 / attempts as f64) / 1000f64);
}

pub fn run_all(do_threaded: bool) -> ErrorResult<()> {
    let paths = glob("problems/*.txt")?;
    let path_bufs = paths.map(|x| x).collect::<Result<Vec<PathBuf>, GlobError>>()?;
    let paths = path_bufs
        .into_iter()
        .map(|x| x.into_os_string().into_string().map_err(|_| SimpleError::new("Couldn't convert file path to string")))
        .collect::<Result<Vec<String>, SimpleError>>()?;
    paths.iter().map(|x| run(x, None, do_threaded)).collect()
}

fn run(file_path: &String, maybe_attemps: Option<usize>, do_threaded: bool) -> ErrorResult<()> {
    let mut params = parse_input(file_path)?;

    debug!("Container size: {:?}", params.size);
    debug!("Amount of blocks: {:?}", params.block_count);
    debug!("Available blocks are: {:?}", params.block_counter);

    if let Some(attemt_count) = maybe_attemps {
        find_average_time(&mut params, attemt_count, do_threaded);
    } else {
        let (maybe_solution, elapsed_micros) = solve_problem(&mut params, do_threaded);
        info!("Solving took {} ms!", elapsed_micros as f64 / 1000f64);

        if let Some(solution) = maybe_solution {
            debug!("Blocks were placed in the following order:\n{:?}", solution.placed_blocks);
            info!("A solution was found:");
            print_grid(&solution.grid, &params.size, params.block_count);
        } else {
            info!("There is no solution!");
        }
    };
    Ok(())
}

fn print_grid(grid: &Vec<usize>, size: &Vec3, block_count: usize) {
    let mut lines: Vec<String> = Vec::new();
    let str_len = f32::log10((block_count) as f32) as usize;
    for y in 0..size.1 {
        let mut line = String::new();
        for z in 0..size.2 {
            for x in 0..size.0 {
                let num = grid[get_pos_index(&Vec3(x, y, z), size)];
                let stri = match num {
                    0 => "-".to_string(),
                    1 => "G".to_string(),
                    n => (n - 1).to_string(),
                };
                for _ in 0..(str_len + 1 - stri.len()) {
                    line.push(' ');
                }
                if stri == "0" {
                    line.push('G');
                } else if stri == "-1" {
                    line.push(' ');
                    line.push('-');
                } else {
                    line.push_str(&stri);
                }
                line.push(' ');
            }
            line.push(' ');
        }
        lines.push(line);
    }
    for line in lines {
        info!("{}", line);
    }
}

fn parse_input(file_path: &String) -> ErrorResult<ProblemParams> {
    let now = Instant::now();
    let contents = fs::read_to_string(file_path)?;

    let lines: Vec<&str> = contents.lines().collect();

    let size_vec: Vec<usize> = lines[0].split_whitespace().map(str::parse::<usize>).collect::<Result<Vec<usize>, ParseIntError>>()?;
    let size: Vec3 = Vec3::from_vec(&size_vec)?;

    let block_count: usize = str::parse(lines[1])?;

    let block_vecs: Vec<Vec<usize>> = lines[2..].iter().map(|&l| l.split_whitespace().map(str::parse::<usize>).collect()).collect::<Result<Vec<Vec<usize>>, ParseIntError>>()?;
    let blocks: Vec<Block> = block_vecs.iter().map(Block::from_vec).collect::<Result<Vec<Block>, SimpleError>>()?;

    if blocks.len() != block_count {
        bail!("Block Amount doesn't match");
    }

    let mut unique_blocks: Vec<Block> = blocks.clone();
    unique_blocks.sort();
    unique_blocks.dedup();
    unique_blocks.reverse();

    debug!("Unique block order: {:?}", unique_blocks);

    let mut block_counter: HashMap<Block, usize> = HashMap::with_capacity(unique_blocks.len());

    blocks.iter().for_each(|&block| *block_counter.entry(block).or_insert(0) += 1);

    let mut block_rotations: HashMap<Block, Vec<Block>> = HashMap::new();

    unique_blocks.iter().for_each(|x| drop(block_rotations.insert(*x, Block::get_rotations(x))));

    let elapsed_micros = now.elapsed().as_micros();
    info!("Parsing took {} ms!", elapsed_micros as f64 / 1000f64);

    return Ok(ProblemParams {
        size,
        block_count,
        unique_blocks,
        block_counter,
        block_rotations,
    });
}
