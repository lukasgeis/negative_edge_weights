cd $(dirname "$0")
cd ../..

cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p "$OUTPUT/roads/"

echo "round,rate,initial,degree" > "$OUTPUT/roads.out"

while getopts r: flag
do
    case "${flag}" in
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10}
do
    # Luxembourg Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/m_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/u_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/z_$NUM.out" &
done

wait

cat $OUTPUT/roads/* >> "$OUTPUT/roads.out"

rm -r "$OUTPUT/roads"

