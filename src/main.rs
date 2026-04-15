use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::time::Instant;
use clap::Parser;
use num_format::{Locale, ToFormattedString};

#[derive(Parser, Debug)]
#[command(name = "Bogo sort benchmark")]
#[command(about = "benchmark your computer by running bogo-sort algorithm")]
struct Args {
    #[arg(short = 'l', long = "length", default_value_t = 50_000_000, help = "The length of the array to be sorted")]
    length: i32,

    #[arg(short = 's', long = "stress", help = "keep on shuffling array until break manually")]
    stress: bool,

    #[arg(short = 'r', long = "norate", help = "DO NOT show realtime rate and ETA in progressbar")]
    rate: bool
}

pub fn advanced_round(value: f64, decimal: i32) -> f64 {
    let factor = 10_f64.powi(decimal);
    (value * factor).round() / factor
}

fn format_rate(v: f64) -> String {
    if v >= 1_000_000.0 {
        format!("{:.3}M/s", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.3}K/s", v / 1_000.0)
    } else {
        format!("{:.0}/s", v)
    }
}

// merge arrays functions
fn merge_two(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let mut res = Vec::with_capacity(a.len() + b.len());

    let (mut i, mut j) = (0, 0);

    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            res.push(a[i]);
            i += 1;
        } else {
            res.push(b[j]);
            j += 1;
        }
    }

    res.extend_from_slice(&a[i..]);
    res.extend_from_slice(&b[j..]);

    res
}

fn parallel_merge(mut vecs: Vec<Vec<f64>>) -> Vec<f64> {


    while vecs.len() > 1 {
        let pb = ProgressBar::new((vecs.len() as u64) / 2);
        pb.set_style(
            ProgressStyle::with_template(
    "[{bar:50.green/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("=> "),
        );
        vecs = vecs
            .par_chunks(2)
            .map(|chunk| {
                pb.inc(1);

                if chunk.len() == 2 {
                    merge_two(chunk[0].clone(), chunk[1].clone())
                } else {
                    chunk[0].clone()
                }
            })
            .collect();
    }

//    pb.finish();

    vecs.into_iter().next().unwrap_or_default()
}

// array operations
fn valid_order(a: &Vec<f64>) -> bool {
    if a.is_empty() {
        return true;
    }
    let mut last = a[0];
    for i in a.iter() {
        if *i >= last {
            last = *i;
        }
        else {
            return false
        }
    }

    true
}
fn valid_order_pb(a: &Vec<f64>) -> bool {
    if a.is_empty() {
        return true;
    }
    let pb = ProgressBar::new(a.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
"[{bar:50.green/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    let mut last = a[0];
    for i in a.iter() {
        pb.inc(1);
        if *i >= last {
            last = *i;
        }
        else {
            return false
        }
    }

    true
}

fn split_vec(v: Vec<f64>, bs: i64) -> Vec<Vec<f64>> {
    let total_chunks = (v.len() + 4) / (bs as usize);
    let pb = ProgressBar::new(total_chunks as u64);
    pb.set_style(
        ProgressStyle::with_template(
"[{bar:50.green/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("=> "),
    );

    v.par_chunks(bs as usize)
        .map(|chunk| {
            pb.inc(1);
            chunk.to_vec()
        })
        .collect()
}
fn bogosort_array(mut a: Vec<f64>) -> Vec<f64> {
    while !valid_order(&a) {
        fastrand::shuffle(&mut a);
    }
    a
}
fn generate_vec(len: usize) -> Vec<f64> {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::with_template(
"[{bar:50.green/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    (0..len)
        .into_par_iter()
        .map(|i| {
            if i % 10 == 0 {
                pb.inc(10);
            }
            //pb.inc(1);
            (len - i) as f64
        })
        .collect()
}

// main
#[tokio::main]
async fn main() {
    let args = Args::parse();

    let blocksize = 7;
    let arrlength = args.length;

    println!("generating array");
    let la = generate_vec(arrlength as usize);

    let start = Instant::now();

    println!("spliting array");
    let arrays = split_vec(la, blocksize as i64);

    println!("sorting splited arrays");

    if args.stress {
        loop {
            let pb = ProgressBar::new(arrays.len() as u64);
            if !args.rate {
                pb.set_style(
                ProgressStyle::with_template(
                    "[{bar:50.green/blue}] {percent}% eta:{eta} {msg}",
                    )
                    .unwrap()
                    .progress_chars("=> "),
                );
            } else {
                pb.set_style(
                ProgressStyle::with_template(
                    "[{bar:50.green/blue}] {percent}%",
                    )
                    .unwrap()
                    .progress_chars("=> "),
                );
            }
            let _: Vec<Vec<f64>> = arrays.clone()
                .into_par_iter()
                .map(|array| {
                    pb.inc(1);
                    pb.set_message(format_rate(pb.per_sec() as f64));
                    bogosort_array(array)
                })
                .collect();
        }
    } else {
            let pb = ProgressBar::new(arrays.len() as u64);
            if !args.rate {
                pb.set_style(
                ProgressStyle::with_template(
                    "[{bar:50.green/blue}] {percent}% eta:{eta} {msg}",
                    )
                    .unwrap()
                    .progress_chars("=> "),
                );
            } else {
                pb.set_style(
                ProgressStyle::with_template(
                    "[{bar:50.green/blue}] {percent}%",
                    )
                    .unwrap()
                    .progress_chars("=> "),
                );
            }
        let start_sorting = Instant::now();
        let x: Vec<Vec<f64>> = arrays
            .into_par_iter()
            .map(|array| {
                pb.inc(1);
                pb.set_message(format_rate(pb.per_sec() as f64));
                bogosort_array(array)
            })
            .collect();
        let sorting_elapsed = start_sorting.elapsed().as_millis();
        println!("merging splited arrays");
        let result: Vec<f64> = parallel_merge(x);
        let total_elapsed = start.elapsed().as_millis();
        
        println!("validating");
        let valid = valid_order_pb(&result);
        if valid {
            println!("No errors occurred during sorting");
        } else {
            eprintln!("Error: Sorted array is still not ordered");
        }
        println!("\n----- Benchmark completed -----");
        
        print!("Total Duration: {}ms, Sorting Duration: {}ms", total_elapsed, sorting_elapsed);
        let splited_arrays_num = ((arrlength as f64) / (blocksize as f64)).ceil();
        let ass = splited_arrays_num / ((sorting_elapsed as f64) / 1000.0);
        let fass = ((advanced_round(ass, 3).floor()) as i64).to_formatted_string(&Locale::en);

        let tss = splited_arrays_num / ((total_elapsed as f64) / 1000.0);
        let ftss = ((advanced_round(tss, 3).floor()) as i64).to_formatted_string(&Locale::en);
        
        println!("sorted arrays: {}\nsplited array sorting speed: {} arrays/s\noverall full array sorting speed: {} arrays/s",splited_arrays_num, fass, ftss);
    }

}
