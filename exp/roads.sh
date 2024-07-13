cd $(dirname "$0")

for EXP in "acceptance" "insertions" "intervals"
do
    make roads -C $EXP
    wait
done
