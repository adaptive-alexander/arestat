#[inline(always)]
pub fn chunk_reqs(requests: usize, threads: usize, thread_nr: usize) -> usize {
    let mut reqs = requests / threads;
    let modulus = requests % threads;
    if modulus != 0 && thread_nr < modulus {
        reqs += 1;
    }
    reqs
}

#[inline(always)]
pub fn print_stats(total_time: u128, timers: Vec<u128>, requests: usize) {
    print_histogram(timers.clone());
    println!("{:<25}{}", "Total time:", fmt_time(total_time));
    println!("{:<25}{}", "Avg time per req:", fmt_time(total_time / requests as u128));
    println!("{:<25}{:.2} req/s", "Requests per second:", req_per_sec(total_time, requests));
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
fn print_histogram(mut timers: Vec<u128>) {
    const N_BARS: f64 = 200.0;
    timers.sort();

    let p05 = timers[(timers.len() as f64 * 0.05) as usize];
    let p95 = timers[(timers.len() as f64 * 0.95) as usize];
    let p99 = timers[(timers.len() as f64 * 0.99) as usize];

    let (min, max) = (timers.iter().min().unwrap().clone(), timers.iter().max().unwrap().clone());
    let range = p95 - p05;
    let bin_size = ((range as f64) / 10.0).ceil() as u128;

    let bar_scale_factor = N_BARS / timers.len() as f64;
    let time_scale_factor = match p95 {
        0..=10_000 => 1,
        10_001..=10_000_000 => 1_000,
        _ => 1_000_000
    };
    let time_unit = match time_scale_factor {
        1 => "ns",
        1_000 => "µs",
        1_000_000 => "ms",
        _ => "something went terribly wrong"
    };

    let mut bins = Vec::with_capacity(timers.len());
    for _ in 0..10 {
        bins.push(vec![])
    }

    let mut idx: usize = 0;

    for t in timers {
        if t < p05 {
            continue;
        }
        if t > p05 + (idx as u128 + 1) * bin_size {
            idx += 1;
            if idx == 10 {
                break;
            }
        }
        bins[idx].push(t);
    }

    let bin_ranges = (0..10).into_iter().map(|v| p05 + v * bin_size).collect::<Vec<_>>();

    // --- Draw histogram
    println!("{:^35}","Histogram of P5 - P95");

    // Set ranges for histogram bin text
    for (i, bin) in bins.iter().enumerate() {
        let range = if i == bins.len() - 1 {
            format!("{} - {} {}", bin_ranges[i] / time_scale_factor, (bin_ranges[i] + bin_size) / time_scale_factor, time_unit)
        } else {
            format!("{} - {} {}", bin_ranges[i] / time_scale_factor, bin_ranges[i + 1] / time_scale_factor, time_unit)
        };
        println!("{}", format!("{:<25}{:|<2$}", range, "", (bin.iter().count() as f64 * bar_scale_factor) as usize));
    }

    println!("\n{:^35}", "Statistics");
    println!("{:<25}{} {}", "P95:", p95 / time_scale_factor, time_unit);
    println!("{:<25}{} {}", "P99:", p99 / time_scale_factor, time_unit);
    println!("{:<25}{}, {} {}", "min, max:", min / time_scale_factor, max / time_scale_factor, time_unit);
}

#[inline(always)]
fn req_per_sec(start: u128, requests: usize) -> f64 {
    requests as f64 / (start as f64 / 1_000_000_000.0)
}
