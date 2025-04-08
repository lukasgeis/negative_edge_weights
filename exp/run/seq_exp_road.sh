#!/bin/bash
#SBATCH --job-name=rnewroad
#SBATCH --partition=general1
#SBATCH --nodes=1 
#SBATCH --ntasks=40
#SBATCH --cpus-per-task=1
#SBATCH --mem-per-cpu=4000
#SBATCH --time=240:00:00
#SBATCH --no-requeue
#SBATCH --mail-type=FAIL
#SBATCH --extra-node-info=2:20:1

# Build binary
cargo build --release --bin seq_exp
BINARY="./target/release/seq_exp"


# Create output directory and temp folders
OUTPUTDIR="/scratch/memhierarchy/geis/rnew/road"
for TEMP in "log" "ins" "pot" "weight"
do
    mkdir -p "$OUTPUTDIR/$TEMP"
done

# Rename old data file
for EXP in "log" "ins" "pot" "weight"
do
    mv "$OUTPUTDIR/$EXP.csv" "$OUTPUTDIR/$EXP.old.csv"
done

function create_outpaths() {
    eval "road_out=''"
    for EXP in "log" "ins" "pot" "weight"
    do
        eval "road_out+='${OUTPUTDIR}/${EXP}/road_${INITIAL}_${NUM}.csv '"
    done
}

# Generate data
ROUNDS=100000000
BFSKIP=1000 
for NUM in {0..10}
do
    for INITIAL in "m" "u" "z"
    do
        # Create Outpaths
        create_outpaths

        $BINARY $road_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL file -p "graphs/luxembourg-contracted.edges" &
    done
done

wait

# Concatenate all files into a big file
for EXP in "log" "ins" "pot" "weight"
do
    head -n 1 "$OUTPUTDIR/$EXP/road_m_0.csv" > "$OUTPUTDIR/$EXP.csv"
    for FILE in $OUTPUTDIR/$EXP/*.csv
    do
        tail -n +2 $FILE >> "$OUTPUTDIR/$EXP.csv"
    done
    rm -r "$OUTPUTDIR/$EXP"
done

