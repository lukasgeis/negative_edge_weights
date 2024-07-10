cd $(dirname "$0")
cd ../..

cargo build --release --features insertions

OUTPUT="data/insertions"

mkdir -p $OUTPUT

for GEN in "gnp" "rhg" "dsf"
do
    mkdir -p "$OUTPUT/$GEN"     
done


HEADER="insertions,acc,algo"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

while getopts n:r: flag
do
    case "${flag}" in
        n) NODES=${OPTARG};;
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10} 
do
    for GEN in "gnp" "rhg"
    do
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd $GEN -n $NODES -d 10 >> "$OUTPUT/$GEN/bd_$NUM.out" &
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d $GEN -n $NODES -d 10 >> "$OUTPUT/$GEN/d_$NUM.out" &
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bf $GEN -n $NODES -d 10 >> "$OUTPUT/$GEN/bf_$NUM.out" &
    done

    # Fix Degree 10
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd --scc dsf -n $(($NODES * 265 / 100)) -d 5.3 >> "$OUTPUT/dsf/bd_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d --scc dsf -n $(($NODES * 265 / 100)) -d 5.3 >> "$OUTPUT/dsf/bd_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bf --scc dsf -n $(($NODES * 265 / 100)) -d 5.3 >> "$OUTPUT/dsf/bd_$NUM.out" &
done

wait

for GEN in "gnp" "rhg" "dsf"
do
    cat $OUTPUT/${GEN}/* >> "$OUTPUT/$GEN.out"
    rm -r "$OUTPUT/$GEN"
done
