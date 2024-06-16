cd $(dirname "$0")
cd ../..

cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p $OUTPUT

for GEN in "gnp" "rhg" "dsf"
do
    mkdir -p "$OUTPUT/$GEN"     
done


HEADER="round,rate,initial,degree"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"

while getopts n:r: flag
do
    case "${flag}" in
        n) NODES=${OPTARG};;
        r) ROUNDS_BASE=${OPTARG};;
    esac
done

for NUM in {1..10} 
do
    for DEGREE in 10 20 50
    do 
        ROUNDS=$(($ROUNDS_BASE * 50 / $DEGREE))

        for GEN in "gnp" "rhg" "dsf"
        do
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/m_${DEGREE}_$NUM.out" & 
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/z_${DEGREE}_$NUM.out" & 
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/u_${DEGREE}_$NUM.out" & 
        done
    done
done

wait

for GEN in "gnp" "rhg" "dsf"
do
    cat $OUTPUT/${GEN}/* >> "$OUTPUT/$GEN.out"
    rm -r "$OUTPUT/$GEN"
done
