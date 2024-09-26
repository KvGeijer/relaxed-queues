import subprocess
import re
import matplotlib.pyplot as plt


def run_command(prefill, producer_threads, consumer_threads, duration):
    cmd = f"cargo r -- --prefill {prefill} --producer-threads {producer_threads} --consumer-threads {consumer_threads} --duration {duration}"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.stdout


def parse_output(output):
    throughput = float(re.search(r"throughput: ([\d.]+)", output).group(1))
    enqueues = int(re.search(r"number of enqueues: (\d+)", output).group(1))
    dequeues = int(re.search(r"number of dequeues: (\d+)", output).group(1))
    return throughput, enqueues, dequeues


def run_benchmark():
    thread_counts = [2, 4, 8, 16, 32]
    results = []

    for thread_count in thread_counts:
        output = run_command(1000, thread_count, thread_count, 2)
        throughput, enqueues, dequeues = parse_output(output)
        results.append((thread_count, throughput, enqueues, dequeues))

    return results


def plot_results(results):
    thread_counts, throughputs, enqueues, dequeues = zip(*results)

    plt.figure(figsize=(12, 8))

    plt.subplot(2, 1, 1)
    plt.plot(thread_counts, throughputs, marker='o')
    plt.title('Throughput vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Throughput')
    plt.xscale('log', base=2)

    plt.subplot(2, 1, 2)
    plt.plot(thread_counts, enqueues, marker='o', label='Enqueues')
    plt.plot(thread_counts, dequeues, marker='o', label='Dequeues')
    plt.title('Enqueues and Dequeues vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Count')
    plt.xscale('log', base=2)
    plt.legend()

    plt.tight_layout()
    plt.savefig('benchmark_results.png')
    plt.close()


if __name__ == "__main__":
    results = run_benchmark()
    plot_results(results)
    print("Benchmark completed. Results saved in 'benchmark_results.png'.")
