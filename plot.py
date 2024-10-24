import subprocess
import re
import matplotlib.pyplot as plt


def run_command(prefill, producer_threads, consumer_threads, duration, queue_name):
    cmd = f"cargo r -- --prefill {prefill} --producer-threads {producer_threads} --consumer-threads {consumer_threads} --duration {duration} --queue-name {queue_name}"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.stdout


def parse_output(output):
    throughput = float(re.search(r"throughput: ([\d.]+)", output).group(1))
    enqueues = int(re.search(r"number of enqueues: (\d+)", output).group(1))
    dequeues = int(re.search(r"number of dequeues: (\d+)", output).group(1))
    return throughput, enqueues, dequeues


def run_benchmark():
    thread_counts = [1, 2, 4, 8]
    queue_types = ['ms-queue', 'dra-queue']
    results = {}

    for queue_type in queue_types:
        queue_results = []
        print(f"\nRunning benchmarks for {queue_type}...")
        for thread_count in thread_counts:
            print(f"Testing with {thread_count} threads...")
            output = run_command(1000, thread_count, thread_count, 2, queue_type)
            throughput, enqueues, dequeues = parse_output(output)
            queue_results.append((thread_count, throughput, enqueues, dequeues))
        results[queue_type] = queue_results
    return results


def plot_results(results):
    plt.figure(figsize=(15, 10))

    # Plot throughput
    plt.subplot(2, 1, 1)
    for queue_type, queue_results in results.items():
        thread_counts, throughputs, _, _ = zip(*queue_results)
        plt.plot(thread_counts, throughputs, marker='o', label=queue_type)
    plt.title('Throughput vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Throughput (ops/sec)')
    plt.xscale('log', base=2)
    plt.grid(True, which="both", ls="-", alpha=0.2)
    plt.legend()

    # Plot enqueues and dequeues
    plt.subplot(2, 1, 2)
    for queue_type, queue_results in results.items():
        thread_counts, _, enqueues, dequeues = zip(*queue_results)
        plt.plot(thread_counts, enqueues, marker='o', label=f'{queue_type} Enqueues')
        plt.plot(thread_counts, dequeues, marker='s', label=f'{queue_type} Dequeues')

    plt.title('Enqueues and Dequeues vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Count')
    plt.xscale('log', base=2)
    plt.grid(True, which="both", ls="-", alpha=0.2)
    plt.legend()

    plt.tight_layout()
    plt.savefig('queue_comparison_results.png')
    plt.close()


if __name__ == "__main__":
    results = run_benchmark()
    plot_results(results)
    print("\nBenchmark completed. Results saved in 'queue_comparison_results.png'.")

    # Print numerical results
    print("\nNumerical Results:")
    for queue_type, queue_results in results.items():
        print(f"\n{queue_type} Results:")
        print("Threads\tThroughput\tEnqueues\tDequeues")
        for thread_count, throughput, enqueues, dequeues in queue_results:
            print(f"{thread_count}\t{throughput:.2f}\t{enqueues}\t{dequeues}")
