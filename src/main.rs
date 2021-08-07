use benchmark::{Benchmark, ITERATIONS};
use scoped_threadpool::Pool;
use std::time::Instant;

const DIM: usize = 10_000;
const P: usize = 4;

fn main() {
    let matrix = build_random_u8_square_matrix_nested();
    count_odds_row_major_traversal(&matrix);
    count_odds_col_major_traversal(&matrix);

    let matrix = build_random_u8_square_matrix_inline();
    count_odds_sequential(&matrix);
    count_odds_parallel(&matrix);
    count_odds_parallel_optimized(&matrix);
}

fn build_random_u8_square_matrix_nested() -> Vec<Vec<u8>> {
    (0..DIM)
        .map(|_| (0..DIM).map(|_| rand::random()).collect())
        .collect()
}

fn build_random_u8_square_matrix_inline() -> Vec<u8> {
    (0..DIM * DIM).map(|_| rand::random()).collect()
}

#[allow(clippy::needless_range_loop)]
fn count_odds_row_major_traversal(matrix: &[Vec<u8>]) {
    let mut benchmark = Benchmark::new("count_odds_row_major_traversal");
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let mut odds = 0;
        for r in 0..DIM {
            for c in 0..DIM {
                if matrix[r][c] % 2 != 0 {
                    odds += 1;
                }
            }
        }
        benchmark.add(odds, t0.elapsed());
    }
    benchmark.print();
}

#[allow(clippy::needless_range_loop)]
fn count_odds_col_major_traversal(matrix: &[Vec<u8>]) {
    let mut benchmark = Benchmark::new("count_odds_col_major_traversal");
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let mut odds = 0;
        for c in 0..DIM {
            for r in 0..DIM {
                if matrix[r][c] % 2 != 0 {
                    odds += 1;
                }
            }
        }
        benchmark.add(odds, t0.elapsed());
    }
    benchmark.print();
}

fn count_odds_sequential(matrix: &[u8]) {
    let mut benchmark = Benchmark::new("count_odds_sequential");
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let mut odds = 0;
        for i in 0..DIM {
            for j in 0..DIM {
                if matrix[i * DIM + j] % 2 != 0 {
                    odds += 1;
                }
            }
        }
        benchmark.add(odds, t0.elapsed());
    }
    benchmark.print();
}

fn count_odds_parallel(matrix: &[u8]) {
    let mut pool = Pool::new(P as u32);
    let mut benchmark = Benchmark::new("count_odds_parallel");
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let mut results = [0; P];
        pool.scoped(|scope| {
            for (p, results_p) in results.iter_mut().enumerate() {
                scope.execute(move || {
                    let chunk_size = DIM / P + 1;
                    let my_start = p * chunk_size;
                    let my_end = std::cmp::min(my_start + chunk_size, DIM);
                    for i in my_start..my_end {
                        for j in 0..DIM {
                            if matrix[i * DIM + j] % 2 != 0 {
                                *results_p += 1;
                            }
                        }
                    }
                });
            }
        });
        let odds = results.iter().sum();
        benchmark.add(odds, t0.elapsed());
    }
    benchmark.print();
}

fn count_odds_parallel_optimized(matrix: &[u8]) {
    let mut pool = Pool::new(P as u32);
    let mut benchmark = Benchmark::new("count_odds_parallel_optimized");
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let mut results = [0; P];
        pool.scoped(|scope| {
            for (p, results_p) in results.iter_mut().enumerate() {
                scope.execute(move || {
                    let mut odds = 0;
                    let chunk_size = DIM / P + 1;
                    let my_start = p * chunk_size;
                    let my_end = std::cmp::min(my_start + chunk_size, DIM);
                    for i in my_start..my_end {
                        for j in 0..DIM {
                            if matrix[i * DIM + j] % 2 != 0 {
                                odds += 1;
                            }
                        }
                    }
                    *results_p = odds;
                });
            }
        });
        let odds = results.iter().sum();
        benchmark.add(odds, t0.elapsed());
    }
    benchmark.print();
}
