cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p $OUTPUT

HEADER="round,rate,initial"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

NODES=1000

for NUM in {1..10} 
do
    # Maximum Weight
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i m gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i m rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i m dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    # Zero
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i z gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i z rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i z dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    # Uniform
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i u gnp -n $NODES -d 10 >> "$OUTPUT/gnp.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i u rhg -n $NODES -d 10 >> "$OUTPUT/rhg.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r 20 -t f64 -i u dsf -n $NODES -d 10 >> "$OUTPUT/dsf.out"

    echo "Round $NUM Done" 
done
