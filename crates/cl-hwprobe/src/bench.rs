//! Single-thread micro-benchmark ("simulation" test).
//!
//! It interprets a small made-up bytecode program: fetch, decode, branch,
//! integer arithmetic and memory accesses, the work a CPU interpreter or a
//! JIT dispatcher does all the time. The score is relative: 1000 is the
//! reference machine defined by `REFERENCE_NS`. Scores are only compared
//! with each other and with the calibration table in `cl-perf`; they are
//! not an industry benchmark.

use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::time::Instant;

/// Duration of one pass on the reference machine (score 1000), release build.
/// Provisional value: a 2.8 GHz cloud Xeon core runs a pass in ~105 ms and
/// scores ~600; recent desktop CPUs are expected around 1000–1400. To be
/// refined with real machines, see docs/prototype/PERFORMANCE.*.md.
const REFERENCE_NS: f64 = 63_000_000.0;
const PASSES: usize = 3;
const STEPS: u32 = 40_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    /// Relative single-thread score (higher is faster).
    pub score: u32,
    /// Best pass, in milliseconds.
    pub best_ms: f64,
}

#[derive(Clone, Copy)]
enum Op {
    Add(u8, u8),
    Xor(u8, u8),
    Load(u8, u8),
    Store(u8, u8),
    Shl(u8),
    JnzBack(u8, u8),
    Inc(u8),
}

fn program() -> Vec<Op> {
    use Op::*;
    vec![
        Inc(0),
        Add(1, 0),
        Load(2, 1),
        Xor(2, 1),
        Store(2, 0),
        Shl(3),
        Add(3, 2),
        Load(4, 3),
        Xor(5, 4),
        Store(5, 1),
        Add(6, 5),
        JnzBack(6, 10),
    ]
}

fn run_once(steps: u32) -> u32 {
    let code = program();
    let mut mem = vec![0u32; 4096];
    let mut r = [1u32; 8];
    let mut pc = 0usize;
    for _ in 0..steps {
        match code[pc] {
            Op::Add(a, b) => r[a as usize] = r[a as usize].wrapping_add(r[b as usize]),
            Op::Xor(a, b) => r[a as usize] ^= r[b as usize].rotate_left(5),
            Op::Load(a, b) => r[a as usize] = mem[(r[b as usize] & 4095) as usize],
            Op::Store(a, b) => mem[(r[b as usize] & 4095) as usize] = r[a as usize],
            Op::Shl(a) => r[a as usize] = r[a as usize].wrapping_shl(1) | (r[a as usize] >> 31),
            Op::Inc(a) => r[a as usize] = r[a as usize].wrapping_add(1),
            Op::JnzBack(a, n) => {
                // Data-dependent branch: hard to predict, like guest code.
                if r[a as usize] & 3 != 0 {
                    pc = pc + 1 - n as usize;
                    continue;
                }
            }
        }
        pc = (pc + 1) % code.len();
    }
    r.iter().fold(0, |acc, v| acc ^ v)
}

pub fn run_cpu_bench() -> BenchResult {
    // Warm-up so the CPU leaves its idle frequency.
    black_box(run_once(black_box(STEPS / 4)));
    let mut best = f64::MAX;
    for _ in 0..PASSES {
        let t = Instant::now();
        black_box(run_once(black_box(STEPS)));
        best = best.min(t.elapsed().as_nanos() as f64);
    }
    BenchResult {
        score: (REFERENCE_NS / best * 1000.0).round() as u32,
        best_ms: best / 1e6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter_is_deterministic() {
        assert_eq!(run_once(10_000), run_once(10_000));
    }
}
