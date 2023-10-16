use colored::Colorize;

#[derive(Default)]
pub struct Stats {
    total_time: u128,
    timers: Vec<u128>,
    requests: usize,
    avg_time_per_request: u128,
    req_per_sec: f64,
    min: u128,
    max: u128,
    p05: u128,
    p95: u128,
    p99: u128,
    bins: Vec<Vec<u128>>,
    bin_lower_range: Vec<u128>,
    bin_size: u128,
}

impl Stats {
    pub fn new(total_time: u128, timers: Vec<u128>, requests: usize) -> Self {
        let mut slf = Self {
            total_time,
            timers,
            requests,
            ..Default::default()
        };

        slf.get_stats();
        slf
    }
    fn get_stats(&mut self) {
        self.avg_time_per_request = self.total_time / self.requests as u128;
        self.req_per_sec = self.requests as f64 / (self.total_time as f64 / 1_000_000_000.0);

        self.timers.sort();

        // Percentiles
        self.p05 = self.timers[(self.timers.len() as f64 *0.05).round() as usize];
        self.p95 = self.timers[(self.timers.len() as f64 * 0.95).round() as usize];
        self.p99 = self.timers[(self.timers.len() as f64 * 0.99).round() as usize];

        // Ranges
        (self.min, self.max) = (*self.timers.iter().min().unwrap(), *self.timers.iter().max().unwrap());
        let range = self.p95 - self.p05;
        self.bin_size = ((range as f64) / 10.0).ceil() as u128;

        // --- Sort into bins
        let mut bins = Vec::with_capacity(self.timers.len());
        for _ in 0..10 {
            bins.push(vec![])
        }

        let mut idx: usize = 0;

        for t in self.timers.clone() {
            if t < self.p05 {
                continue;
            }
            if t > self.p05 + (idx as u128 + 1) * self.bin_size {
                idx += 1;
                if idx == 10 {
                    break;
                }
            }
            bins[idx].push(t);
        }

        self.bins = bins;

        self.bin_lower_range = (0..10).map(|v| self.p05 + v * self.bin_size).collect::<Vec<_>>();
    }
    pub fn print(&self) {
        const N_BARS: f64 = 200.0;

        let bar_scale_factor = N_BARS / self.timers.len() as f64;
        let time_scale_factor = match self.p95 {
            0..=10_000 => 1,
            10_001..=10_000_000 => 1_000,
            10_000_001..=10_000_000_000 => 1_000_000,
            _ => 1_000_000_000
        };
        let time_unit = match time_scale_factor {
            1 => "ns",
            1_000 => "µs",
            1_000_000 => "ms",
            1_000_000_000 => "s",
            _ => "something went terribly wrong"
        };

        // --- Draw histogram
        println!("{:^35}", "Histogram of P5 - P95".blue());

        // Set ranges for histogram bin text
        for (i, bin) in self.bins.iter().enumerate() {
            let range = if i == self.bins.len() - 1 {
                format!("{} - {} {}", self.bin_lower_range[i] / time_scale_factor + 1, (self.bin_lower_range[i] + self.bin_size) / time_scale_factor, time_unit)
            } else if i == 0 {
                format!("{} - {} {}", self.bin_lower_range[i] / time_scale_factor, self.bin_lower_range[i + 1] / time_scale_factor, time_unit)
            } else {
                format!("{} - {} {}", self.bin_lower_range[i] / time_scale_factor + 1, self.bin_lower_range[i + 1] / time_scale_factor, time_unit)
            };
            println!("{:<25}{:|<2$}", range, "", (bin.len() as f64 * bar_scale_factor) as usize);
        }

        println!("\n{:^35}", "Statistics".blue());
        println!("{:<25}{}", "Total time:", fmt_time(self.total_time));
        println!("{:<25}{:.2} req/s", "Requests per sec:", self.req_per_sec);
        println!("{:<25}{} {}", "Avg time per request", self.avg_time_per_request / time_scale_factor, time_unit);
        println!("{:<25}{} {}", "P95:", self.p95 / time_scale_factor, time_unit);
        println!("{:<25}{} {}", "P99:", self.p99 / time_scale_factor, time_unit);
        println!("{:<25}{}, {} {}", "min, max:", self.min / time_scale_factor, self.max / time_scale_factor, time_unit);
    }
}

#[inline(always)]
fn fmt_time(total_time: u128) -> String {
    let elapsed = total_time / 1_000;

    match elapsed {
        0..=10_000 => { format!("{} µs", elapsed) }
        10_001..=10_000_000 => { format!("{} ms", elapsed / 1_000) }
        10_000_001..=60_000_000 => { format!("{}.{} s", elapsed / 1_000_000, (elapsed % 1_000_000) / 1000) }
        _ => { format!("{}:{}.{} ", elapsed / 60_000_000, (elapsed % 60_000_000) / 1_000_000, (elapsed % 1_000_000) / 1000) }
    }
}

#[inline(always)]
pub fn chunk_reqs(requests: usize, threads: usize, thread_nr: usize) -> usize {
    let mut reqs = requests / threads;
    let modulus = requests % threads;
    if modulus != 0 && thread_nr < modulus {
        reqs += 1;
    }
    reqs
}
