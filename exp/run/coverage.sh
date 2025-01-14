#!/bin/bash
#SBATCH --job-name=rnewcoverage
#SBATCH --partition=general1
#SBATCH --nodes=1 
#SBATCH --ntasks=80
#SBATCH --cpus-per-task=1
#SBATCH --mem-per-cpu=4000
#SBATCH --time=100:00:00
#SBATCH --no-requeue
#SBATCH --mail-type=FAIL

cargo build --release --bin coverage
BINARY="./target/release/coverage"

OUTPUTDIR="/scratch/memhierarchy/geis/rnew/coverage"
mkdir -p $OUTPUTDIR

COMMON="--steps 40 --runs 10 --quit-early"

# Cycle
$BINARY $COMMON --nodes 8 --repetitions 500 --constraints Cycle 2>> "${OUTPUTDIR}/cycle8.json"
$BINARY $COMMON --nodes 12 --repetitions 200 --constraints Cycle 2>> "${OUTPUTDIR}/cycle12.json"
$BINARY $COMMON --nodes 16 --repetitions 100 --constraints Cycle 2>> "${OUTPUTDIR}/cycle16.json"

# NestedCycles
$BINARY $COMMON --nodes 8 --repetitions 500 --constraints NestedCycles 2>> "${OUTPUTDIR}/nested8.json"
$BINARY $COMMON --nodes 12 --repetitions 200 --constraints NestedCycles 2>> "${OUTPUTDIR}/nested12.json"
$BINARY $COMMON --nodes 16 --repetitions 100 --constraints NestedCycles 2>> "${OUTPUTDIR}/nested16.json"

# PairedCycles
$BINARY $COMMON --nodes 8 --repetitions 500 --constraints PairedCycles 2>> "${OUTPUTDIR}/paired8.json"
$BINARY $COMMON --nodes 12 --repetitions 200 --constraints PairedCycles 2>> "${OUTPUTDIR}/paired12.json"
$BINARY $COMMON --nodes 16 --repetitions 100 --constraints PairedCycles 2>> "${OUTPUTDIR}/paired16.json"


