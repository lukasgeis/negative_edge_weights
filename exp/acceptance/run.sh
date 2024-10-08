cd $(dirname "$0")
cd ../..

cargo build --release --features acceptance

OUTPUT="data/acceptance"

mkdir -p $OUTPUT

for GEN in "gnp" "rhg" "dsf" "roads"
do
    mkdir -p "$OUTPUT/$GEN"     
done


HEADER="round,rate,initial,degree"

echo $HEADER > "$OUTPUT/gnp.out"
echo $HEADER > "$OUTPUT/rhg.out"
echo $HEADER > "$OUTPUT/dsf.out"
echo $HEADER > "$OUTPUT/roads.out"

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

        for GEN in "gnp" "rhg"
        do
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i m $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/m_${DEGREE}_$NUM.out" & 
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i z $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/z_${DEGREE}_$NUM.out" & 
            ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS -t f64 -i u $GEN -n $NODES -d $DEGREE >> "$OUTPUT/$GEN/u_${DEGREE}_$NUM.out" & 
        done
    
    done

    # Degree 10
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5)) -t f64 -i m --scc dsf -n $(($NODES * 25 / 10)) -d 6 >> "$OUTPUT/dsf/m_10_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5)) -t f64 -i z --scc dsf -n $(($NODES * 25 / 10)) -d 6 >> "$OUTPUT/dsf/u_10_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5)) -t f64 -i u --scc dsf -n $(($NODES * 25 / 10)) -d 6 >> "$OUTPUT/dsf/z_10_$NUM.out" &

    # Degree 20
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5 / 2)) -t f64 -i m --scc dsf -n $(($NODES * 2)) -d 14 >> "$OUTPUT/dsf/m_20_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5 / 2)) -t f64 -i z --scc dsf -n $(($NODES * 2)) -d 14 >> "$OUTPUT/dsf/u_20_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS_BASE * 5 / 2)) -t f64 -i u --scc dsf -n $(($NODES * 2)) -d 14 >> "$OUTPUT/dsf/z_20_$NUM.out" &

    # Degree 50
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS_BASE -t f64 -i m --scc dsf -n $(($NODES * 17 / 10)) -d 47 >> "$OUTPUT/dsf/m_50_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS_BASE -t f64 -i z --scc dsf -n $(($NODES * 17 / 10)) -d 47 >> "$OUTPUT/dsf/u_50_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $ROUNDS_BASE -t f64 -i u --scc dsf -n $(($NODES * 17 / 10)) -d 47 >> "$OUTPUT/dsf/z_50_$NUM.out" &

    # Luxembourg Contracted
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS * 5)) -t f64 -i m file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/m_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS * 5)) -t f64 -i u file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/u_$NUM.out" &
    ./target/release/random_negative_weights -w=-100 -W 100 -r $(($ROUNDS * 5)) -t f64 -i z file -p "exp/roads_data/luxembourg-contracted.edges" >> "$OUTPUT/roads/z_$NUM.out" &
done

wait

for GEN in "gnp" "rhg" "dsf" "roads"
do
    cat $OUTPUT/${GEN}/* >> "$OUTPUT/$GEN.out"
    rm -r "$OUTPUT/$GEN"
done
