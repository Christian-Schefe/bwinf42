use std::{collections::HashMap, error::Error};

use simple_error::{bail, SimpleError};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Block(pub usize, pub usize, pub usize);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Vec3(pub usize, pub usize, pub usize);

impl Vec3 {
    pub fn from_vec(vec: &Vec<usize>) -> Result<Vec3, SimpleError> {
        if vec.len() == 3 {
            Ok(Vec3(vec[0], vec[1], vec[2]))
        } else {
            bail!("Couldn't convert Vec to Vec3: len of vec is not 3")
        }
    }
}

impl Block {
    pub fn from_vec(vec: &Vec<usize>) -> Result<Block, SimpleError> {
        if vec.len() == 3 {
            Ok(Block(vec[0], vec[1], vec[2]))
        } else {
            bail!("Couldn't convert Vec to Block: len of vec is not 3")
        }
    }

    pub fn get_rotations(self: &Block) -> Vec<Block> {
        if self.0 == self.1 && self.1 == self.2 {
            vec![Block(self.0, self.1, self.2)]
        } else if self.0 == self.1 || self.1 == self.2 || self.0 == self.2 {
            vec![Block(self.0, self.1, self.2), Block(self.1, self.2, self.0), Block(self.2, self.0, self.1)]
        } else {
            vec![
                Block(self.0, self.1, self.2),
                Block(self.0, self.2, self.1),
                Block(self.1, self.0, self.2),
                Block(self.1, self.2, self.0),
                Block(self.2, self.0, self.1),
                Block(self.2, self.1, self.0),
            ]
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProblemParams {
    pub size: Vec3,
    pub block_count: usize,
    pub unique_blocks: Vec<Block>,
    pub block_counter: HashMap<Block, usize>,
    pub block_rotations: HashMap<Block, Vec<Block>>,
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub grid: Vec<usize>,
    pub placed_blocks: Vec<(Block, Vec3)>,
}

pub type ErrorResult<T> = Result<T, Box<dyn Error>>;
