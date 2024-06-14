DATA_DIR := ../../data/intervals
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})


all: clean run plot

run:
	bash run.sh

plot:
	mkdir -p ${PLOT_DIR}
	python3 plot.py

clean:
	rm -r ${DATA_DIR}
	rm -r ${PLOT_DIR}
