pub mod item;
pub mod solution;
pub mod solver;

#[derive(Debug)]
pub enum Delta {
    Decrease(u64),
    Increase(u64),
    Keep,
}
