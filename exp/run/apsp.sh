#!/bin/bash
#SBATCH --job-name=rnewapsp
#SBATCH --partition=general1
#SBATCH --nodes=1 
#SBATCH --ntasks=40
#SBATCH --cpus-per-task=1
#SBATCH --mem-per-cpu=2000
#SBATCH --time=100:00:00
#SBATCH --no-requeue
#SBATCH --mail-type=FAIL
#SBATCH --extra-node-info=2:20:1

# Build binary
cargo build --release --bin apsp
BINARY="./target/release/apsp"


# Create output directory and temp folders
OUTPUTDIR="/scratch/memhierarchy/geis/rnew/apsp"
mkdir -p $OUTPUTDIR

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
ROUNDS=1000000000
for NUM in {0..100}
do
    for INITIAL in "m" "u" "z"
    do

        # Degree 10
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL gnp -n 10000 -d 10 >> "${OUTPUTDIR}/gnp_10.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL rhg -n 10000 -d 10 >> "${OUTPUTDIR}/rhg_10.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 dsf -n 25000 -d 6 >> "${OUTPUTDIR}/dsf_10.csv" &

        # Degree 20
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL gnp -n 10000 -d 20 >> "${OUTPUTDIR}/gnp_20.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL rhg -n 10000 -d 20 >> "${OUTPUTDIR}/rhg_20.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 dsf -n 20000 -d 14 >> "${OUTPUTDIR}/dsf_20.csv" &

        # Degree 50
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL gnp -n 10000 -d 50 >> "${OUTPUTDIR}/gnp_50.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL rhg -n 10000 -d 50 >> "${OUTPUTDIR}/rhg_50.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 dsf -n 17000 -d 47 >> "${OUTPUTDIR}/dsf_50.csv" &

        # Degree 500
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL gnp -n 10000 -d 500 >> "${OUTPUTDIR}/gnp_500.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL rhg -n 10000 -d 500 >> "${OUTPUTDIR}/rhg_500.csv" &
        job_limit `nproc`
        $BINARY -w=-100 -W 100 -r $ROUNDS -i $INITIAL --scc --mult 10 dsf -n 12800 -d 1350 >> "${OUTPUTDIR}/dsf_500.csv" &
    done
done

wait
