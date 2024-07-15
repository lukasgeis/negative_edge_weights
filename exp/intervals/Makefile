DATA_DIR := ../../data/intervals
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})

.PHONY: all run plot test roads testroads clean

all: clean run plot

run:
	bash run.sh -n 10000 -r 100

plot:
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py; \
	)

test: clean
	bash run.sh -n 1000 -r 10
	mkdir -p ${PLOT_DIR}/test
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py test; \
	) 
	
roads:
	bash roads.sh -r 100

testroads:
	bash roads.sh -r 1

clean:
	rm -r ${DATA_DIR}
