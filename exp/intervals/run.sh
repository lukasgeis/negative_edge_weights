cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p $OUTPUT

HEADER="round,avg,frac,time"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

for NUM in {1..10} 
do
    ./target/release/random_negative_weights -w=-100 -W 100 -r 100 -t f64 --bidir gnp -n 10000 -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 100 -t f64 --bidir rhg -n 10000 -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 100 -t f64 --bidir dsf -n 10000 -d 10 >> "$OUTPUT/dsf.out"
    echo "Round $NUM Done"
done

