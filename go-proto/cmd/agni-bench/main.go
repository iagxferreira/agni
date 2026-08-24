package main

import (
	"bufio"
	"flag"
	"fmt"
	"net"
	"os"
	"sort"
	"sync"
	"time"

	"agni-go/internal/wire"
)

type benchResult struct {
	totalOps  int
	elapsed   time.Duration
	latencies []time.Duration
}

func (r benchResult) opsPerSec() float64 {
	return float64(r.totalOps) / r.elapsed.Seconds()
}

func (r benchResult) percentile(p float64) time.Duration {
	sorted := append([]time.Duration(nil), r.latencies...)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i] < sorted[j] })
	idx := int(float64(len(sorted)) * p / 100.0)
	if idx >= len(sorted) {
		idx = len(sorted) - 1
	}
	return sorted[idx]
}

func sendRecv(r *bufio.Reader, w *bufio.Writer, cmd string) error {
	if err := wire.WriteFrame(w, []byte(cmd)); err != nil {
		return err
	}
	if err := w.Flush(); err != nil {
		return err
	}
	_, err := wire.ReadFrame(r)
	return err
}

// runScenario mirrors BenchRunner.kt: each worker holds a persistent TCP
// connection and sends requests sequentially, no reconnects per operation.
func runScenario(host string, port int, concurrency, ops int, buildCmd func(int) string) (benchResult, error) {
	opsPerTask := ops / concurrency
	start := time.Now()

	var wg sync.WaitGroup
	results := make([][]time.Duration, concurrency)
	errs := make([]error, concurrency)

	for c := 0; c < concurrency; c++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()

			conn, err := net.Dial("tcp", net.JoinHostPort(host, fmt.Sprint(port)))
			if err != nil {
				errs[idx] = err
				return
			}
			defer conn.Close()

			r := bufio.NewReader(conn)
			w := bufio.NewWriter(conn)

			latencies := make([]time.Duration, 0, opsPerTask)
			for i := 0; i < opsPerTask; i++ {
				opStart := time.Now()
				if err := sendRecv(r, w, buildCmd(i)); err != nil {
					errs[idx] = err
					return
				}
				latencies = append(latencies, time.Since(opStart))
			}
			results[idx] = latencies
		}(c)
	}

	wg.Wait()

	for _, err := range errs {
		if err != nil {
			return benchResult{}, err
		}
	}

	var all []time.Duration
	for _, l := range results {
		all = append(all, l...)
	}

	return benchResult{totalOps: len(all), elapsed: time.Since(start), latencies: all}, nil
}

func printResult(label string, r benchResult) {
	fmt.Println()
	fmt.Printf("=== %s ===\n", label)
	fmt.Printf("  Ops:          %d\n", r.totalOps)
	fmt.Printf("  Total time:   %.2fs\n", r.elapsed.Seconds())
	fmt.Printf("  Throughput:   %.0f ops/sec\n", r.opsPerSec())
	fmt.Printf("  Latency p50:  %s\n", r.percentile(50))
	fmt.Printf("  Latency p95:  %s\n", r.percentile(95))
	fmt.Printf("  Latency p99:  %s\n", r.percentile(99))
}

func main() {
	host := flag.String("host", "127.0.0.1", "server host")
	port := flag.Int("port", 6379, "server port")
	concurrency := flag.Int("c", 50, "number of concurrent connections")
	ops := flag.Int("n", 10000, "total number of operations per scenario")
	flag.Parse()

	fmt.Println("=== Agni Bench (Go) ===")
	fmt.Printf("  Target:       %s:%d\n", *host, *port)
	fmt.Printf("  Concurrency:  %d connections\n", *concurrency)
	fmt.Printf("  Ops/scenario: %d\n", *ops)

	run := func(label string, ops int, buildCmd func(int) string) benchResult {
		result, err := runScenario(*host, *port, *concurrency, ops, buildCmd)
		if err != nil {
			fmt.Printf("%s scenario failed: %v\n", label, err)
			os.Exit(1)
		}
		return result
	}

	// Warm up
	run("warm-up", *concurrency, func(int) string { return "PING" })

	printResult("PING", run("PING", *ops, func(int) string { return "PING" }))

	printResult("SET (1000 unique keys)", run("SET", *ops, func(i int) string {
		return fmt.Sprintf("SET key:%d value:%d", i%1000, i)
	}))

	printResult("GET (hit)", run("GET hit", *ops, func(i int) string {
		return fmt.Sprintf("GET key:%d", i%1000)
	}))

	printResult("GET (miss)", run("GET miss", *ops, func(i int) string {
		return fmt.Sprintf("GET missing:%d", i)
	}))

	fmt.Println()
	fmt.Println("=== Mixed SET+GET ===")
	mixed := run("Mixed", *ops, func(i int) string {
		if i%2 == 0 {
			return fmt.Sprintf("SET key:%d value:%d", i%500, i)
		}
		return fmt.Sprintf("GET key:%d", i%500)
	})
	fmt.Printf("  Ops:          %d\n", mixed.totalOps)
	fmt.Printf("  Total time:   %.2fs\n", mixed.elapsed.Seconds())
	fmt.Printf("  Throughput:   %.0f ops/sec\n", mixed.opsPerSec())

	fmt.Println()
	fmt.Println("Done.")
}
