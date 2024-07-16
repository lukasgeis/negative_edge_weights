DATA_DIR := ../../data/intervals
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})

.PHONY: run plot test roads testroads

run:
	bash run.sh -n 10000 -r 100

plot:
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py; \
	)

test: 
	bash run.sh -n 1000 -r 10
	mkdir -p ${PLOT_DIR}/test
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py test; \
	) 
	
roads:
	bash roads.sh -r 200

testroads:
	bash roads.sh -r 1

