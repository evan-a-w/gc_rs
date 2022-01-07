cd ~/dev/gc
cargo run
valgrind -q --leak-check=full ~/dev/gc/target/debug/gc_rs_bin
