DATA_DIR := ../../data/intervals
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})

.PHONY: all run plot test clean

all: clean run plot

run:
	bash run.sh -n 10000 -d 10 -r 100

plot:
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py; \
	)

test: clean
	bash run.sh -n 1000 -d 10 -r 10
	mkdir -p ${PLOT_DIR}/test
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py test; \
	) 
	
clean:
	rm -r ${DATA_DIR}
