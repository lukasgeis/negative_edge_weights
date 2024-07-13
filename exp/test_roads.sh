cd $(dirname "$0")

for EXP in "acceptance" "insertions" "intervals"
do
    make testroads -C $EXP
    wait
done
