cargo build --release

OUTPUT="data/mini_bench"
mkdir -p $OUTPUT

for SEED in 1 12 123 1234 12345
do
    echo "[SEED] $SEED" >> "$OUTPUT/onedir.out"
    echo "[SEED] $SEED" >> "$OUTPUT/twodir.out"
    ./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 -s $SEED gnp -n 2000 -d 10 >> "$OUTPUT/onedir.out"
    ./target/release/random_negative_weights -r 10 -w=-3 -W 10 -t f64 -s $SEED --bidir gnp -n 2000 -d 10 >> "$OUTPUT/twodir.out"  
    echo "" >> "$OUTPUT/onedir.out"
    echo "" >> "$OUTPUT/twodir.out"
 done
