import subprocess
import re
import matplotlib.pyplot as plt
import argparse
from typing import Dict, List, Tuple, Optional

def parse_args():
    parser = argparse.ArgumentParser(description='Benchmark different queue implementations')
    parser.add_argument('--relaxed', nargs=2, type=int, metavar=('QUEUE_NUMBER', 'D_CHOICE'),
                       help='Run DRA queue tests with specified number of subqueues and d-choice parameter')
    return parser.parse_args()

def run_command(prefill: int, producer_threads: int, consumer_threads: int, 
                duration: int, queue_config: str) -> str:
    cmd = f"cargo r -- --prefill {prefill} --producer-threads {producer_threads} " \
          f"--consumer-threads {consumer_threads} --duration {duration} {queue_config}"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error running command: {cmd}")
        print(f"Error output: {result.stderr}")
        raise RuntimeError("Command failed")
    return result.stdout

def parse_output(output: str) -> Tuple[float, int, int]:
    throughput = float(re.search(r"throughput: ([\d.]+)", output).group(1))
    enqueues = int(re.search(r"number of enqueues: (\d+)", output).group(1))
    dequeues = int(re.search(r"number of dequeues: (\d+)", output).group(1))
    return throughput, enqueues, dequeues

def get_queue_configs(relaxed_config: Optional[Tuple[int, int]] = None) -> Dict[str, str]:
    if relaxed_config:
        subqueues, d_choice = relaxed_config
        base_queues = {
            'MSQueue': 'ms-queue',
            'LockFreeQueue': 'lock-free-queue',
            'CrossbeamQueue': 'crossbeam-queue',
            'ConcurrentQueue': 'concurrent-queue'
        }
        return {
            f"DRA-{name}-{subqueues}-{d_choice}": f"dra-queue --subqueue {queue} --subqueues {subqueues} --choice {d_choice}"
            for name, queue in base_queues.items()
        }
    else:
        return {
            'MSQueue': 'ms-queue',
            'LockFreeQueue': 'lock-free-queue',
            'CrossbeamQueue': 'crossbeam-queue',
            'ConcurrentQueue': 'concurrent-queue'
        }

def run_benchmark(relaxed_config: Optional[Tuple[int, int]] = None) -> Dict[str, List[Tuple[int, float, int, int]]]:
    thread_counts = [1, 2, 4, 8, 16]  # Extended thread counts
    queue_configs = get_queue_configs(relaxed_config)
    results = {}
    
    for queue_name, queue_config in queue_configs.items():
        queue_results = []
        print(f"\nRunning benchmarks for {queue_name}...")
        
        for thread_count in thread_counts:
            print(f"Testing with {thread_count} threads...")
            try:
                output = run_command(1024, thread_count, thread_count, 2, queue_config)
                throughput, enqueues, dequeues = parse_output(output)
                queue_results.append((thread_count, throughput, enqueues, dequeues))
            except Exception as e:
                print(f"Error running benchmark: {e}")
                continue
                
        results[queue_name] = queue_results
    
    return results

def plot_results(results: Dict[str, List[Tuple[int, float, int, int]]]):
    plt.figure(figsize=(15, 10))
    colors = plt.cm.tab10(np.linspace(0, 1, len(results)))
    
    # Plot throughput
    plt.subplot(2, 1, 1)
    for (queue_type, queue_results), color in zip(results.items(), colors):
        if not queue_results:  # Skip if no results
            continue
        thread_counts, throughputs, _, _ = zip(*queue_results)
        plt.plot(thread_counts, throughputs, marker='o', label=queue_type, color=color)
    
    plt.title('Throughput vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Throughput (ops/sec)')
    plt.xscale('log', base=2)
    plt.grid(True, which="both", ls="-", alpha=0.2)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    
    # Plot enqueues and dequeues
    plt.subplot(2, 1, 2)
    for (queue_type, queue_results), color in zip(results.items(), colors):
        if not queue_results:  # Skip if no results
            continue
        thread_counts, _, enqueues, dequeues = zip(*queue_results)
        plt.plot(thread_counts, enqueues, marker='o', label=f'{queue_type} Enqueues', 
                color=color, linestyle='-')
        plt.plot(thread_counts, dequeues, marker='s', label=f'{queue_type} Dequeues', 
                color=color, linestyle='--')
    
    plt.title('Enqueues and Dequeues vs. Thread Count')
    plt.xlabel('Number of Threads (Producer and Consumer)')
    plt.ylabel('Count')
    plt.xscale('log', base=2)
    plt.grid(True, which="both", ls="-", alpha=0.2)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    
    plt.tight_layout()
    plt.savefig('queue_comparison_results.png', bbox_inches='tight')
    plt.close()

def print_numerical_results(results: Dict[str, List[Tuple[int, float, int, int]]]):
    print("\nNumerical Results:")
    for queue_type, queue_results in results.items():
        if not queue_results:  # Skip if no results
            continue
        print(f"\n{queue_type} Results:")
        print("Threads\tThroughput\tEnqueues\tDequeues")
        for thread_count, throughput, enqueues, dequeues in queue_results:
            print(f"{thread_count}\t{throughput:.2f}\t{enqueues}\t{dequeues}")

if __name__ == "__main__":
    args = parse_args()
    relaxed_config = (args.relaxed[0], args.relaxed[1]) if args.relaxed else None
    
    if relaxed_config:
        print(f"Running DRA queue benchmarks with {relaxed_config[0]} subqueues "
              f"and d-choice={relaxed_config[1]}")
    else:
        print("Running standard queue benchmarks")
    
    results = run_benchmark(relaxed_config)
    plot_results(results)
    print_numerical_results(results)
    print("\nBenchmark completed. Results saved in 'queue_comparison_results.png'.")
