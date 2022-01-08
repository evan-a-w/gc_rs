cd ~/dev/gc_rs
cargo build --bin leak_check
valgrind -q --leak-check=full ~/dev/gc_rs/target/debug/leak_check
cargo build --release --bin leak_check
valgrind -q --leak-check=full ~/dev/gc_rs/target/release/leak_check
