#!/bin/bash
#SBATCH --job-name=rnewbatch
#SBATCH --partition=general1
#SBATCH --nodes=2 
#SBATCH --ntasks=80
#SBATCH --cpus-per-task=1
#SBATCH --mem-per-cpu=4000
#SBATCH --time=240:00:00
#SBATCH --no-requeue
#SBATCH --mail-type=FAIL
#SBATCH --extra-node-info=2:20:1

# Build binary
cargo build --release --bin batching
BINARY="./target/release/batching"


# Create output directory and temp folders
OUTPUTDIR="/scratch/memhierarchy/geis/rnew"
mkdir -p "$OUTPUTDIR"
OUTPUT="$OUTPUTDIR/batching.csv"

# Rename old data file
mv "$OUTPUTDIR/batching.csv" "$OUTPUTDIR/batching.old.csv"

echo "graph,n,deg,initial,rounds,seq,time" > $OUTPUT

# Generate data
ROUNDS=100000000
for NUM in {1..10}
do
    for INITIAL in "m" "u" "z"
    do
        for SEQ in "" "--seq"
        do 
            # Degree 10
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ gnp -n 10000 -d 10 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ rhg -n 10000 -d 10 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 $SEQ dsf -n 25000 -d 6 >> $OUTPUT

            # Degree 20
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ gnp -n 10000 -d 20 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ rhg -n 10000 -d 20 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 $SEQ dsf -n 20000 -d 14 >> $OUTPUT

            # Degree 50
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ gnp -n 10000 -d 50 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL $SEQ rhg -n 10000 -d 50 >> $OUTPUT
            $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 $SEQ dsf -n 17000 -d 47 >> $OUTPUT
        done
    done
done

