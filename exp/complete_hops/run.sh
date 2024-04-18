mkdir -p data/complete_hops

cargo build --release --features hops

./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 complete -n 100 > "data/complete_hops/100_1_1.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 complete -n 100 > "data/complete_hops/100_2_5.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 complete -n 100 > "data/complete_hops/100_3_10.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 complete -n 1000 > "data/complete_hops/1000_1_1.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 complete -n 1000 > "data/complete_hops/1000_2_5.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 complete -n 1000 > "data/complete_hops/1000_3_10.out" &

wait
