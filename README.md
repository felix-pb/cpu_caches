# CPU Caches and Why You Care

Rust translation of the C++ examples in Scott Meyers' 2014 talk called ["CPU Caches and Why You Care"](https://www.youtube.com/watch?v=WDIkqP4JbkE)

## Exhibit 1

Consider this function to count the number of odd elements in a square matrix of size `DIM` by `DIM`:

```rust
fn count_odds_row_major_traversal(matrix: &[Vec<u8>]) -> u32 {
    let mut odds = 0;
    for r in 0..DIM {
        for c in 0..DIM {
            if matrix[r][c] % 2 != 0 {
                odds += 1;
            }
        }
    }
    odds
}
```

The function above visits every element row-by-row (i.e. row-major traversal). Now, consider this function that instead visits every element column-by-column (i.e. column-major traversal):

```diff
    fn count_odds_col_major_traversal(matrix: &[Vec<u8>]) -> u32 {
        let mut odds = 0;
!       for c in 0..DIM {
!           for r in 0..DIM {
                if matrix[r][c] % 2 != 0 {
                    odds += 1;
                }
            }
        }
        odds
    }
```

Both functions visit every element exactly once, and they both perform the same number of reads and writes. However, on my macOS laptop, I get the following results for `DIM` = 10_000 (i.e. a 100MB matrix):

```
[count_odds_row_major_traversal]
avg = 52.22233ms
mid = 52.595066ms
min = 50.999287ms
max = 53.521421ms

[count_odds_col_major_traversal]
avg = 920.038508ms
mid = 922.493244ms
min = 896.868147ms
max = 944.147035ms
```

That's right, the row-major traversal is more than 17 times faster than the column-major traversal. As you might have guessed, this is because row-major traversal better utilizes CPU caches. You can watch Scott Meyers' talk if you'd like to understand why.

We can further improve our utilization of CPU caches by storing the matrix as a single `Vec<u8>` instead of a `Vec<Vec<u8>>`:

```rust
fn count_odds_sequential(matrix: &[u8]) -> u32 {
    let mut odds = 0;
    for i in 0..DIM {
        for j in 0..DIM {
            if matrix[i * DIM + j] % 2 != 0 {
                odds += 1;
            }
        }
    }
    odds
}
```

On my macOS laptop, I get a 17% improvement in execution time compared to the row-major traversal above:

```
[count_odds_sequential]
avg = 43.272157ms
mid = 42.422754ms
min = 40.888349ms
max = 48.806561ms
```

## Exhibit 2

Now, let's say that we want to parallelize our program. Here's one possible implementation with [scoped-threadpool](https://docs.rs/scoped_threadpool/), where `P` is the number of parallel worker threads:

```rust
fn count_odds_parallel(matrix: &[u8]) -> u32 {
    let mut pool = Pool::new(P as u32);
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
    odds
}
```

My macOS laptop has 4 cores, so if I run this with `P` = 4, I get the following results:

```
[count_odds_parallel]
avg = 111.844857ms
mid = 112.632096ms
min = 107.300084ms
max = 115.004631ms
```

Hmmm... now it takes more than twice as long to count the number of odd elements compared to the seqential algorithm. Again, the key is CPU caches! This time, the problem is that the `results` array is most likely stored entirely on the same cache line. When an element is updated by one thread with `*results_p += 1`, that cache line is marked as dirty on all other cores.

To avoid this, we can store the number of odd elements in each chunk in a local stack variable:

```diff
    fn count_odds_parallel_optimized(matrix: &[u8]) -> u32 {
        let mut pool = Pool::new(P as u32);
        let mut results = [0; P];
        pool.scoped(|scope| {
            for (p, results_p) in results.iter_mut().enumerate() {
                scope.execute(move || {
!                   let mut odds = 0;
                    let chunk_size = DIM / P + 1;
                    let my_start = p * chunk_size;
                    let my_end = std::cmp::min(my_start + chunk_size, DIM);
                    for i in my_start..my_end {
                        for j in 0..DIM {
                            if matrix[i * DIM + j] % 2 != 0 {
!                               odds += 1;
                            }
                        }
                    }
!                   *results_p = odds;
                });
            }
        });
        let odds = results.iter().sum();
        odds
    }
```

In this case, each thread is storing `odds` in its own stack frame. Those stack frames won't be on the same cache line, thus solving the cache invalidation problem. Now, when compared to the sequential results (average ≃ 43 ms), we get near-linear improvements with the number of threads:

```
[count_odds_parallel_optimized] P = 2
avg = 23.107787ms
mid = 22.585558ms
min = 21.881104ms
max = 25.181685ms

[count_odds_parallel_optimized] P = 4
avg = 12.796829ms
mid = 12.934181ms
min = 11.865153ms
max = 13.637674ms
```
