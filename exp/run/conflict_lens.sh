#!/bin/bash
#SBATCH --job-name=rnew
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
cargo build --release --bin conflict_lens
BINARY="./target/release/conflict_lens"


# Create output directory and temp folders
OUTPUTDIR="/scratch/memhierarchy/geis/rnew"
mkdir -p "$OUTPUTDIR/conflicts"

# Rename old data file
mv "$OUTPUTDIR/conflicts.csv" "$OUTPUTDIR/conflicts.old.csv"

# Functions to generate output paths
function create_outpaths() {
    create_gen_outpath "gnp"
    create_gen_outpath "rhg"
    create_gen_outpath "dsf"
}
function create_gen_outpath() {
    for DEG in 10 20 50
    do
        eval "$1_${DEG}_out='--log ${OUTPUTDIR}/conflicts/$1_${DEG}_${INITIAL}_${NUM}.csv '"
    done
}

# Generate data
ROUNDS=100000000
for NUM in {0..10}
do
    for INITIAL in "m" "u" "z"
    do
        # Create Outpaths
        create_outpaths

        # Degree 10
        $BINARY $gnp_10_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 gnp -n 10000 -d 10 &
        $BINARY $rhg_10_out  -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 rhg -n 10000 -d 10 &
        $BINARY $dsf_10_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 -t i64 dsf -n 25000 -d 6 &

        # Degree 20
        $BINARY $gnp_20_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 gnp -n 10000 -d 20 &
        $BINARY $rhg_20_out  -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 rhg -n 10000 -d 20 &
        $BINARY $dsf_20_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 -t i64 dsf -n 20000 -d 14 &

        # Degree 50
        $BINARY $gnp_50_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 gnp -n 10000 -d 50 &
        $BINARY $rhg_50_out  -w=-100 -W 100 -r $ROUNDS -i $INITIAL -t i64 rhg -n 10000 -d 50 &
        $BINARY $dsf_50_out -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 -t i64 dsf -n 17000 -d 47 &            
    done
done

wait

# Concatenate all files into a big file
head -n 1 "$OUTPUTDIR/conflicts/gnp_10_m_0.csv" > "$OUTPUTDIR/conflicts.csv"
for FILE in $OUTPUTDIR/conflicts/*.csv
do
    tail -n +2 $FILE >> "$OUTPUTDIR/conflicts.csv"
done
rm -r "$OUTPUTDIR/conflicts"
