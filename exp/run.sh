cd $(dirname "$0")

for EXP in "acceptance" "cycledist" "insertions" "intervals"
do
    make run -C $EXP
    wait
done


for EXP in "acceptance" "cycledist" "insertions" "intervals"
do
    make plot -C $EXP &
done

wait
