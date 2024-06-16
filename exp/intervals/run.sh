cd $(dirname "$0")
cd ../..

cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p $OUTPUT

HEADER="round,avg,frac,time,algo"

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
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd gnp -n $NODES -d $DEGREE >> "$OUTPUT/$GEN.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d gnp -n $NODES -d $DEGREE >> "$OUTPUT/$GEN.out"
    done
done
