cd $(dirname "$0")
cd ../..

cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p $OUTPUT

HEADER="round,avg,frac,time,algo"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

NODES=10000
DEGREE=10
ROUNDS=100

for NUM in {1..10} 
do
    # BiDijkstra
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd gnp -n $NODES -d $DEGREE >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd rhg -n $NODES -d $DEGREE >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd dsf -n $NODES -d $DEGREE >> "$OUTPUT/dsf.out"

    # Dijkstra
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d gnp -n $NODES -d $DEGREE >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d rhg -n $NODES -d $DEGREE >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d dsf -n $NODES -d $DEGREE >> "$OUTPUT/dsf.out"

done

