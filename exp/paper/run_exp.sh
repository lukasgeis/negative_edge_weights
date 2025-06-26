# Build binary
cargo build --release --bin seq_exp
BINARY="./target/release/seq_exp"


# Create output directory and temp folders
OUTPUTDIR="data"
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
    for DEG in 10 20 50 500
    do
        eval "$1_${DEG}_out=''"
        for EXP in "log" "ins" "pot" "weight"
        do
            eval "$1_${DEG}_out+='${OUTPUTDIR}/${EXP}/$1_${DEG}_${INITIAL}_${NUM}.csv '"
        done
    done
    eval "road_out=''"
    for EXP in "log" "ins" "pot" "weight"
    do
        eval "road_out+='${OUTPUTDIR}/${EXP}/road_${INITIAL}_${NUM}.csv '"
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

        # Degree 10
        job_limit `nproc`
        $BINARY $gnp_10_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL gnp -n 10000 -d 10 &
        job_limit `nproc`
        $BINARY $rhg_10_out  -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL rhg -n 10000 -d 10 &
        job_limit `nproc`
        $BINARY $dsf_10_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL --scc --mult 10 dsf -n 25000 -d 6 &

        # Degree 20
        job_limit `nproc`
        $BINARY $gnp_20_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL gnp -n 10000 -d 20 &
        job_limit `nproc`
        $BINARY $rhg_20_out  -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL rhg -n 10000 -d 20 &
        job_limit `nproc`
        $BINARY $dsf_20_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL --scc --mult 10 dsf -n 20000 -d 14 &

        # Degree 50
        job_limit `nproc`
        $BINARY $gnp_50_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL gnp -n 10000 -d 50 &
        job_limit `nproc`
        $BINARY $rhg_50_out  -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL rhg -n 10000 -d 50 &
        job_limit `nproc`
        $BINARY $dsf_50_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL --scc --mult 10 dsf -n 17000 -d 47 &

        # Degree 500
        job_limit `nproc`
        $BINARY $gnp_500_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL gnp -n 10000 -d 500 &
        job_limit `nproc`
        $BINARY $rhg_500_out  -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL rhg -n 10000 -d 500 &
        job_limit `nproc`
        $BINARY $dsf_500_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL --scc --mult 10 dsf -n 12800 -d 1350 &

        # Road
        job_limit `nproc`
        $BINARY $road_out -w=-100 -W 100 -r $ROUNDS --bf-skip $BFSKIP -i $INITIAL file -p "graphs/luxembourg-contracted.edges" &
    done
done

wait

# Concatenate all files into a big file
for EXP in "log" "ins" "pot" "weight"
do
    head -n 1 "$OUTPUTDIR/$EXP/gnp_10_m_0.csv" > "$OUTPUTDIR/$EXP.csv"
    for FILE in $OUTPUTDIR/$EXP/*.csv
    do
        tail -n +2 $FILE >> "$OUTPUTDIR/$EXP.csv"
    done
    rm -r "$OUTPUTDIR/$EXP"
done

