cd $(dirname "$0")
cd ../..

cargo build --release --features cycle

OUTPUT="data/cycledist"

mkdir -p $OUTPUT

HEADER="round,weight"

echo $HEADER > "$OUTPUT/data.out"

while getopts n: flag
do
    case "${flag}" in
        n) NODES=${OPTARG};;
    esac
done

for NUM in {1..10} 
do
    ./target/release/random_negative_weights -w=-100 -W 100 -t i64 -i z cycle -n $NODES >> "$OUTPUT/data.out"
done
