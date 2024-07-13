cd $(dirname "$0")
cd ../..

cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p "$OUTPUT/roads/be"
mkdir -p "$OUTPUT/roads/bc"

echo "round,rate,initial,degree" > "$OUTPUT/roads-be.out"
echo "round,rate,initial,degree" > "$OUTPUT/roads-bc.out"

while getopts r: flag
do
    case "${flag}" in
        r) ROUNDS=${OPTARG};;
    esac
done

for NUM in {1..10}
do
    # Berlin
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m file -p "exp/roads_data/berlin.edges" >> "$OUTPUT/roads/be/m_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u file -p "exp/roads_data/berlin.edges" >> "$OUTPUT/roads/be/u_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z file -p "exp/roads_data/berlin.edges" >> "$OUTPUT/roads/be/z_$NUM.out" &

    # Berlin Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m file -p "exp/roads_data/berlin-contracted.edges" >> "$OUTPUT/roads/bc/m_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u file -p "exp/roads_data/berlin-contracted.edges" >> "$OUTPUT/roads/bc/u_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z file -p "exp/roads_data/berlin-contracted.edges" >> "$OUTPUT/roads/bc/z_$NUM.out" &
done

wait

cat $OUTPUT/roads/be/* >> "$OUTPUT/roads-be.out"
cat $OUTPUT/roads/bc/* >> "$OUTPUT/roads-bc.out"

rm -r "$OUTPUT/roads"

