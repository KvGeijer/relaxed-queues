# Concurrent Relaxed Lock-Free Queues in Rust

WORK IN PROGRESS.

**This repositories contains:**
- Traits for concurrent queues, including one for queues as components.
- Wrappers around open-source concurrent queues, integrating them with our traits.
- An implementation of a simple [Michael-Scott (MS) Queue](https://www.cs.rochester.edu/~scott/papers/1996_PODC_queues.pdf), which is the most foundational lock-free concurrent queue algorithm.
- A relaxed queue implementation, where items can be dequeued out of order.
- A compilation target to evaluate the performance of a single queue for some parallelism level. Can be chained in a script to compare the scalability of our different queues.

**Goal is to learn about:**
- unsafe rust
- concurrency in rust

## Things we want to try
- Make a similar implementation of a MSQueue in C++ using [Folly](https://github.com/facebook/folly) (C++ library for hazard pointers), do we get close to the same perf? If not, why?
- Implement more strict queues, now based on fetch-and-add, such as LCRQ.
- Implement more relaxed queue variants, such as the round-robin queue.
