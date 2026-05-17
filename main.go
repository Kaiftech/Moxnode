// Moxnode now runs on Rust (faster, swarm mode, self-writing).
// Install Rust, then from this folder:
//
//   cargo run --release
//   cargo run --release -- --once
//   cargo run --release -- --status
//   cargo run --release -- --swarm 1000
//
// +go:build ignore

package main

import "fmt"

func main() {
	fmt.Println("Use: cargo run --release")
	fmt.Println("See main.go comments for flags.")
}
