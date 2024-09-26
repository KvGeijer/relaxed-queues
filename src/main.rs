use clap::Parser;
use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

use relaxed_queues::concurrent_queue::{ms::MSQueue, ConcurrentQueue, Handle};

fn main() {
    let config = BenchConfig::parse();
    let queue = MSQueue::new();
    benchmark_producer_consumer(queue, config);
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct BenchConfig {
    /// number of elements to add to the queue before starting the main
    /// threaded test.
    #[arg(long)]
    prefill: usize,
    /// number of threads pushing elements onto the queue.
    #[arg(long)]
    producer_threads: usize,
    /// number of threads popping elements off the queue.
    #[arg(long)]
    consumer_threads: usize,
    /// duration in seconds to run the test
    #[arg(long)]
    duration: usize,
}

fn benchmark_producer_consumer<C>(queue: C, config: BenchConfig)
where
    C: ConcurrentQueue<i32>,
    for<'a> &'a C: Send,
{
    let mut handle = queue.register();
    for i in 0..config.prefill {
        handle.enqueue(i as i32);
    }

    let done: AtomicBool = AtomicBool::new(false);
    let enqueues = AtomicUsize::new(0);
    let dequeues = AtomicUsize::new(0);

    thread::scope(|s| {
        for _ in 0..config.producer_threads {
            s.spawn(|| {
                let mut local_enqueues = 0;
                let mut handle = queue.register();
                while !done.load(Ordering::Relaxed) {
                    handle.enqueue(405);
                    local_enqueues += 1;
                }
                enqueues.fetch_add(local_enqueues, Ordering::Relaxed);
            });
        }
        for _ in 0..config.consumer_threads {
            s.spawn(|| {
                let mut local_dequeues = 0;
                let mut handle = queue.register();
                while !done.load(Ordering::Relaxed) {
                    handle.dequeue();
                    local_dequeues += 1;
                }
                dequeues.fetch_add(local_dequeues, Ordering::Relaxed);
            });
        }

        std::thread::sleep(Duration::from_secs(config.duration as u64));
        done.store(true, Ordering::Relaxed);
    });

    let enqueues = enqueues.into_inner();
    let dequeues = dequeues.into_inner();
    println!(
        "throughput: {}",
        (enqueues + dequeues) as f64 / config.duration as f64
    );
    println!("number of enqueues: {}", enqueues);
    println!("number of dequeues: {}", dequeues);
}
