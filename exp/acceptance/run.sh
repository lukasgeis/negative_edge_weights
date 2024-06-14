cd $(dirname "$0")
cd ../..

cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p $OUTPUT

HEADER="round,rate,initial,degree"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

NODES=10000
ROUNDS_BASE=1000

for NUM in {1..10} 
do
    for DEGREE in 10 20 50
    do 
        ROUNDS=$(($ROUNDS_BASE * 50 / $DEGREE))

        # Maximum Weight
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m gnp -n $NODES -d $DEGREE >> "$OUTPUT/gnp.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m rhg -n $NODES -d $DEGREE >> "$OUTPUT/rhg.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m dsf -n $NODES -d $DEGREE >> "$OUTPUT/dsf.out"

        # Zero
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z gnp -n $NODES -d $DEGREE >> "$OUTPUT/gnp.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z rhg -n $NODES -d $DEGREE >> "$OUTPUT/rhg.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z dsf -n $NODES -d $DEGREE >> "$OUTPUT/dsf.out"

        # Uniform
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u gnp -n $NODES -d $DEGREE >> "$OUTPUT/gnp.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u rhg -n $NODES -d $DEGREE >> "$OUTPUT/rhg.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u dsf -n $NODES -d $DEGREE >> "$OUTPUT/dsf.out"
    done
done
