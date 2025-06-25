#!/bin/bash
#SBATCH --job-name=rnew
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
OUTPUTDIR="/scratch/memhierarchy/geis/rnew/high_degree"
for TEMP in "log" "ins" "pot" "weight"
do
    mkdir -p "$OUTPUTDIR/$TEMP"
done

# Rename old data file
for EXP in "log" "ins" "pot" "weight"
do
    mv "$OUTPUTDIR/$EXP.csv" "$OUTPUTDIR/$EXP.old.csv"
done

# Functions to generate output paths
function create_outpaths() {
    create_gen_outpath "gnp"
    create_gen_outpath "rhg"
    create_gen_outpath "dsf"
}
function create_gen_outpath() {
    eval "$1_out=''"
    for EXP in "log" "ins" "pot" "weight"
    do
        eval "$1_out+='${OUTPUTDIR}/${EXP}/$1_${INITIAL}_${NUM}.csv '"
    done
}

function job_limit() {
    # Test for single positive integer input
    if (( $# == 1 )) && [[ $1 =~ ^[1-9][0-9]*$ ]]
    then

        # Check number of running jobs
        joblist=($(jobs -rp))
        while (( ${#joblist[*]} >= $(($1 / 2)) ))
        do

            # Wait for any job to finish
            command='wait '${joblist[0]}
            for job in ${joblist[@]:1}
            do
                command+=' || wait '$job
            done
            eval $command
            joblist=($(jobs -rp))
        done
   fi
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

        job_limit `nproc`
        $BINARY $gnp_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL gnp -n 10000 -d 500 &
        job_limit `nproc`
        $BINARY $rhg_out  -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL rhg -n 10000 -d 500 &
        job_limit `nproc`
        $BINARY $dsf_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL --scc --mult 10 dsf -n 12800 -d 1350 &      
    done
done

wait

# Concatenate all files into a big file
for EXP in "log" "ins" "pot" "weight"
do
    head -n 1 "$OUTPUTDIR/$EXP/gnp_m_0.csv" > "$OUTPUTDIR/$EXP.csv"
    for FILE in $OUTPUTDIR/$EXP/*.csv
    do
        tail -n +2 $FILE >> "$OUTPUTDIR/$EXP.csv"
    done
    rm -r "$OUTPUTDIR/$EXP"
done

