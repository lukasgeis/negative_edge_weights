cargo build --release --features insertions

OUTPUT="data/insertions"

mkdir -p $OUTPUT

HEADER="insertions,acc,algo"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

NODES=1000

for NUM in {1..10} 
do
    # BiDijkstra
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bd gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bd rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bd dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    # Dijkstra
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a d gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a d rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a d dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    # BellmanFord
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bf gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bf rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -a bf dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    echo "Round $NUM Done" 
done
