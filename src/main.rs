#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap::{Parser, ValueEnum};
use core_affinity::CoreId;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Barrier,
    },
    thread,
    time::Duration,
};

use relaxed_queues::{
    concurrent_queue::ms::MSQueue, relaxed_queues::dra_queue::DRaQueue, ConcurrentQueue, Handle,
};

fn main() {
    let config = BenchConfig::parse();
    match config.queue_name {
        Queue::DraQueue => {
            benchmark_producer_consumer(DRaQueue::<MSQueue<_>, _>::new(8, 2), config)
        }
        Queue::MSQueue => benchmark_producer_consumer(MSQueue::new(), config),
    };
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct BenchConfig {
    /// number of elements to add to the queue before starting the main
    /// threaded test.
    #[arg(long, default_value_t = 1024)]
    prefill: usize,

    /// number of threads pushing elements onto the queue.
    #[arg(short, long)]
    producer_threads: usize,

    /// number of threads popping elements off the queue.
    #[arg(short, long)]
    consumer_threads: usize,

    /// duration in seconds to run the test
    #[arg(short, long)]
    duration: usize,

    #[arg(short, long, value_enum)]
    queue_name: Queue,
}

#[derive(Clone, ValueEnum)]
pub enum Queue {
    MSQueue,
    DraQueue,
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
    let barrier = Barrier::new(config.producer_threads + config.consumer_threads + 1);

    let available_cores: Vec<CoreId> =
        core_affinity::get_core_ids().unwrap_or(vec![CoreId { id: 0 }]);
    let mut core_iter = available_cores.into_iter().cycle();
    thread::scope(|s| {
        // To get move semantict for thread closures
        let queue = &queue;
        let enqueues = &enqueues;
        let dequeues = &dequeues;
        let done = &done;
        let barrier = &barrier;

        for _ in 0..config.producer_threads {
            let core: CoreId = core_iter.next().unwrap();
            s.spawn(move || {
                core_affinity::set_for_current(core);
                let mut local_enqueues = 0;
                let mut handle = queue.register();
                barrier.wait();
                while !done.load(Ordering::Relaxed) {
                    handle.enqueue(405);
                    local_enqueues += 1;
                }
                enqueues.fetch_add(local_enqueues, Ordering::Relaxed);
            });
        }
        for _ in 0..config.consumer_threads {
            let core: CoreId = core_iter.next().unwrap();
            s.spawn(move || {
                core_affinity::set_for_current(core);
                let mut local_dequeues = 0;
                let mut handle = queue.register();
                barrier.wait();
                while !done.load(Ordering::Relaxed) {
                    handle.dequeue();
                    local_dequeues += 1;
                }
                dequeues.fetch_add(local_dequeues, Ordering::Relaxed);
            });
        }

        barrier.wait();
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
