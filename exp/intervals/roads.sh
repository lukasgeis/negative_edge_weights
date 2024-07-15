cd $(dirname "$0")
cd ../..

cargo build --release --features intervals

OUTPUT="data/intervals"

mkdir -p "$OUTPUT/roads"


echo "round,avg,frac,time,algo"> "$OUTPUT/roads.out"

while getopts r: flag
do
    case "${flag}" in
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10}
do
    # Luxembourg Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a bd file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/bd_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -a d file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/d_$NUM.out" &
done

wait

cat $OUTPUT/roads/* >> "$OUTPUT/roads.out"

rm -r "$OUTPUT/roads"

