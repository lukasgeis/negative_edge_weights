mkdir -p data/sptree_size

cargo build --release --features sptree_size

./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_1_1_f.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_2_5_f.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_3_10_f.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_1_1_f.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_2_5_f.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_3_10_f.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_1_1_i.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_2_5_i.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_3_10_i.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_1_1_i.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_2_5_i.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_3_10_i.out" &

wait 


cargo build --release --features sptree_size,bidir

./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_1_1_f.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_2_5_f.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 gnp -n 100 -d 10 >> "data/sptree_size/100_3_10_f.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_1_1_f.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_2_5_f.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_3_10_f.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_1_1_i.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_2_5_i.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t i64 gnp -n 100 -d 10 >> "data/sptree_size/100_3_10_i.out" &
./target/release/random_negative_weights -r 10 -w=-1 -W 1 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_1_1_i.out" &
./target/release/random_negative_weights -r 10 -w=-2 -W 5 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_2_5_i.out" &
./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t i64 gnp -n 1000 -d 10 >> "data/sptree_size/1000_3_10_i.out" &

wait


