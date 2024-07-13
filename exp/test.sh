cd $(dirname "$0")

for EXP in "acceptance" "cycledist" "insertions" "intervals"
do
    make test -C $EXP
done
