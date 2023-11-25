use std::{collections::HashMap, sync::mpsc, thread, time::Instant};

use log::debug;

use crate::data_structs::{Block, ProblemParams, Solution, Vec3};

pub fn solve_problem(params: &mut ProblemParams, do_threaded: bool) -> (Option<Solution>, u128) {
    let size = params.size;

    let mut grid: Vec<usize> = vec![0; size.0 * size.1 * size.2];
    let mut placed_block_count: usize = 0;
    let mut placed_blocks: Vec<(Block, Vec3)> = Vec::with_capacity(params.block_count);
    let mut block_counter = params.block_counter.clone();

    if !check_volumes(&size, &block_counter) {
        return (None, 0);
    }

    do_place(&mut grid, &Vec3(size.0 / 2, size.1 / 2, size.2 / 2), &Block(1, 1, 1), &size, &mut placed_block_count, &mut placed_blocks);

    let now = Instant::now();

    let has_solution = if do_threaded {
        threaded_search(
            &mut grid,
            &size,
            &mut block_counter,
            &mut placed_block_count,
            params.block_count,
            &params.unique_blocks,
            &params.block_rotations,
            &mut placed_blocks,
        )
    } else {
        dfs(
            &mut grid,
            &size,
            &mut block_counter,
            &mut placed_block_count,
            params.block_count,
            &params.unique_blocks,
            &params.block_rotations,
            &mut placed_blocks,
        )
    };

    let elapsed_micros = now.elapsed().as_micros();

    if has_solution {
        (Some(Solution { grid, placed_blocks }), elapsed_micros)
    } else {
        (None, elapsed_micros)
    }
}

fn dfs(
    grid: &mut Vec<usize>,
    size: &Vec3,
    block_counter: &mut HashMap<Block, usize>,
    placed_block_count: &mut usize,
    block_count: usize,
    unique_blocks: &Vec<Block>,
    block_rotations: &HashMap<Block, Vec<Block>>,
    placed_blocks: &mut Vec<(Block, Vec3)>,
) -> bool {
    let maybe_pos = scan_for_necessary(grid, size);
    if let Some(pos) = maybe_pos {
        for &block in unique_blocks {
            if block_counter[&block] == 0 {
                continue;
            }

            *block_counter.get_mut(&block).unwrap() -= 1;

            for rotation in block_rotations[&block].iter() {
                if is_pos_blocked(grid, &pos, &rotation, size) {
                    continue;
                };

                do_place(grid, &pos, &rotation, size, placed_block_count, placed_blocks);

                if *placed_block_count == block_count + 1 {
                    return true;
                }

                let result = dfs(grid, size, block_counter, placed_block_count, block_count, unique_blocks, block_rotations, placed_blocks);
                if result {
                    return result;
                }

                undo_place(grid, &pos, &rotation, size, placed_block_count, placed_blocks);
            }

            *block_counter.get_mut(&block).unwrap() += 1;
        }
    }
    false
}

fn threaded_search(
    grid_: &mut Vec<usize>,
    size_: &Vec3,
    block_counter_: &mut HashMap<Block, usize>,
    placed_block_count_: &mut usize,
    block_count: usize,
    unique_blocks_: &Vec<Block>,
    block_rotations_: &HashMap<Block, Vec<Block>>,
    placed_blocks_: &mut Vec<(Block, Vec3)>,
) -> bool {
    let pos = Vec3(0, 0, 0);
    let mut threads = Vec::new();
    let (tx, rx) = mpsc::channel();

    for &block in unique_blocks_.iter() {
        let mut grid = grid_.clone();
        let mut block_counter = block_counter_.clone();
        let size = size_.clone();
        let mut placed_block_count = placed_block_count_.clone();
        let unique_blocks = unique_blocks_.clone();
        let block_rotations = block_rotations_.clone();
        let mut placed_blocks = placed_blocks_.clone();
        let thread_tx = tx.clone();
        let handle = thread::spawn(move || {
            *block_counter.get_mut(&block).unwrap() -= 1;

            for rotation in block_rotations[&block].iter() {
                if is_pos_blocked(&mut grid, &pos, &rotation, &size) {
                    continue;
                };

                do_place(&mut grid, &pos, &rotation, &size, &mut placed_block_count, &mut placed_blocks);

                let result = dfs(&mut grid, &size, &mut block_counter, &mut placed_block_count, block_count, &unique_blocks, &block_rotations, &mut placed_blocks);
                if result {
                    let _ = thread_tx.send(Some((grid, placed_blocks)));
                    return;
                }

                undo_place(&mut grid, &pos, &rotation, &size, &mut placed_block_count, &mut placed_blocks);
            }

            *block_counter.get_mut(&block).unwrap() += 1;
            let _ = thread_tx.send(None);
        });
        threads.push(handle);
    }

    for _ in 0..unique_blocks_.len() {
        if let Ok(res) = rx.recv() {
            if let Some((solved_grid, solved_blocks)) = res {
                *grid_ = solved_grid;
                *placed_blocks_ = solved_blocks;
                return true;
            }
        }
    }
    false
}
fn do_place(grid: &mut Vec<usize>, pos: &Vec3, block: &Block, size: &Vec3, block_count: &mut usize, placed_blocks: &mut Vec<(Block, Vec3)>) {
    *block_count += 1;
    placed_blocks.push((*block, *pos));
    for pos in block_to_pos_indices(pos, block, size) {
        grid[pos] = *block_count;
    }
}

fn undo_place(grid: &mut Vec<usize>, pos: &Vec3, block: &Block, size: &Vec3, block_count: &mut usize, placed_blocks: &mut Vec<(Block, Vec3)>) {
    *block_count -= 1;
    placed_blocks.pop();
    for pos in block_to_pos_indices(pos, block, size) {
        grid[pos] = 0;
    }
}

fn scan_for_necessary(grid: &mut Vec<usize>, size: &Vec3) -> Option<Vec3> {
    for x in 0..size.0 {
        for y in 0..size.1 {
            for z in 0..size.2 {
                let pos = Vec3(x, y, z);
                if is_necessary_pos(grid, &pos, size) {
                    return Some(pos);
                }
            }
        }
    }
    None
}

fn is_necessary_pos(grid: &mut Vec<usize>, pos: &Vec3, size: &Vec3) -> bool {
    if is_out_of_bounds(&pos, size) || grid[get_pos_index(&pos, size)] > 0 {
        return false;
    };

    if pos.0 > 0 {
        let left_pos = Vec3(pos.0 - 1, pos.1, pos.2);
        if !is_out_of_bounds(&left_pos, size) && grid[get_pos_index(&left_pos, size)] == 0 {
            return false;
        };
    }

    if pos.1 > 0 {
        let down_pos = Vec3(pos.0, pos.1 - 1, pos.2);
        if !is_out_of_bounds(&down_pos, size) && grid[get_pos_index(&down_pos, size)] == 0 {
            return false;
        };
    }

    if pos.2 > 0 {
        let back_pos = Vec3(pos.0, pos.1, pos.2 - 1);
        if !is_out_of_bounds(&back_pos, size) && grid[get_pos_index(&back_pos, size)] == 0 {
            return false;
        };
    }
    true
}

fn is_pos_blocked(grid: &mut Vec<usize>, pos: &Vec3, block: &Block, size: &Vec3) -> bool {
    for pos in block_to_pos_vec3(pos, block) {
        if is_out_of_bounds(&pos, size) || grid[get_pos_index(&pos, size)] > 0 {
            return true;
        };
    }
    false
}

fn block_to_pos_indices(pos: &Vec3, block: &Block, size: &Vec3) -> Vec<usize> {
    block_to_pos_vec3(pos, block).iter().map(|x| get_pos_index(x, size)).collect()
}

fn block_to_pos_vec3(pos: &Vec3, block: &Block) -> Vec<Vec3> {
    let mut pos_vec: Vec<Vec3> = Vec::with_capacity(block.0 * block.1 * block.2);
    for x in (pos.0)..(pos.0 + block.0) {
        for y in (pos.1)..(pos.1 + block.1) {
            for z in (pos.2)..(pos.2 + block.2) {
                pos_vec.push(Vec3(x, y, z));
            }
        }
    }
    pos_vec
}

pub fn get_pos_index(pos: &Vec3, size: &Vec3) -> usize {
    pos.0 + (pos.1 * size.0) + (pos.2 * size.0 * size.1)
}

fn is_out_of_bounds(pos: &Vec3, size: &Vec3) -> bool {
    pos.0 >= size.0 || pos.1 >= size.1 || pos.2 >= size.2
}

fn check_volumes(size: &Vec3, block_counter: &HashMap<Block, usize>) -> bool {
    let container_vol = size.0 * size.1 * size.2;
    let mut block_vol_sum = 1;
    for (block, count) in block_counter.iter() {
        let block_vol = block.0 * block.1 * block.2;
        block_vol_sum += block_vol * *count;
    }

    let matches = block_vol_sum == container_vol;

    if !matches {
        debug!("Volumes: {} / {}", block_vol_sum, container_vol);
    }
    matches
}
