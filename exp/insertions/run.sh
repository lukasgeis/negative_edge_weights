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

while getopts n:d:r: flag
do
    case "${flag}" in
        n) NODES=${OPTARG};;
        d) DEGREE=${OPTARG};;
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10} 
do
    for GEN in "gnp" "rhg" "dsf"
    do
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd gnp -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/bd_$NUM.out" &
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d gnp -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/d_$NUM.out" &
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bf gnp -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/bf_$NUM.out" &
    done
done

wait

for GEN in "gnp" "rhg" "dsf"
do
    cat $OUTPUT/${GEN}/* >> "$OUTPUT/$GEN.out"
    rm -r "$OUTPUT/$GEN"
done
