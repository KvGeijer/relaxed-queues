# Concurrent Relaxed Lockfree Queues in Rust

WORK IN PROGRESS.

**This repositories contains:**
- Traits for concurrent sub-queues and queues.
- implementation of a simple MSQueue.
- benchmarking scripts
- a relaxed queue implementation

**Goal is to learn about:**
- unsafe rust
- concurrency in rust

## Things we want to try
- Make a similar implementation of a MSQueue in C++ using folly (C++ library for hazard pointers), do we get close to the same perf? If not, why?
- Compare perf of a queue from the crate concurrent_queue/crossbeam-queue/lockfree::queue with our MSQueue.
- Try using concurrent_queue/crossbeam-queue/lockfree::queue as subqueues (impl ConcurrentSubQueue trait).
- Add
- Implement more subqueue variants! (fetch_add LCRQ, etc.)
- Implement more relaxed queue variants.
