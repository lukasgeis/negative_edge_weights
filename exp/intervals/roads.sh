cd $(dirname "$0")
cd ../..

cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p "$OUTPUT/roads/be"
mkdir -p "$OUTPUT/roads/bc"


echo "round,avg,frac,time,algo"> "$OUTPUT/roads-be.out"
echo "round,avg,frac,time,algo"> "$OUTPUT/roads-bc.out"

while getopts r: flag
do
    case "${flag}" in
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10}
do
    # Berlin
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd file -p "exp/roads_data/berlin.edges" >> "$OUTPUT/roads/be/bd_$NUM.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d file -p "exp/roads_data/berlin.edges" >> "$OUTPUT/roads/be/d_$NUM.out"

    # Berlin Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd file -p "exp/roads_data/berlin-contracted.edges" >> "$OUTPUT/roads/bc/bd_$NUM.out"
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d file -p "exp/roads_data/berlin-contracted.edges" >> "$OUTPUT/roads/bc/d_$NUM.out"
done

wait

cat $OUTPUT/roads/be/* >> "$OUTPUT/roads-be.out"
cat $OUTPUT/roads/bc/* >> "$OUTPUT/roads-bc.out"

rm -r "$OUTPUT/roads"

