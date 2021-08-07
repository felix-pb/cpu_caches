use std::time::Duration;

pub const ITERATIONS: usize = 9;

pub struct Benchmark {
    name: &'static str,
    results: Vec<u32>,
    durations: Vec<Duration>,
}

impl Benchmark {
    pub fn new(name: &'static str) -> Benchmark {
        Benchmark {
            name,
            results: Vec::with_capacity(ITERATIONS),
            durations: Vec::with_capacity(ITERATIONS),
        }
    }

    pub fn add(&mut self, result: u32, duration: Duration) {
        self.results.push(result);
        self.durations.push(duration);
    }

    pub fn print(&mut self) {
        assert!(self.results.iter().all(|&r| r == self.results[0]));
        self.durations.sort();
        let len = self.durations.len();
        let avg = self.durations.iter().sum::<Duration>() / len as u32;
        let mid = self.durations.get(len / 2).unwrap();
        let min = self.durations.first().unwrap();
        let max = self.durations.last().unwrap();
        println!("[{}]", self.name);
        println!("avg = {:?}", avg);
        println!("mid = {:?}", mid);
        println!("min = {:?}", min);
        println!("max = {:?}", max);
        println!();
    }
}
