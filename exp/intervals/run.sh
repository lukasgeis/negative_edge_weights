cd $(dirname "$0")
cd ../..

cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p $OUTPUT

HEADER="round,avg,frac,time,algo"

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
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd $GEN -n $NODES -d 10 >> "$OUTPUT/$GEN.out"
        ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d $GEN -n $NODES -d 10 >> "$OUTPUT/$GEN.out"
    done

    # Fix Degree 10
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd --scc dsf -n $(($NODES * 25 / 10)) -d 6 >> "$OUTPUT/dsf.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d --scc dsf -n $(($NODES * 25 / 10)) -d 6 >> "$OUTPUT/dsf.out"

    # Luxembourg Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS * 2)) -t f64 -a bd file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS * 2)) -t f64 -a d file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads.out"
done
