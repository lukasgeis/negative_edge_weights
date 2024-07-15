DATA_DIR := ../../data/cycledist
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})

.PHONY: run plot test

run:
	bash run.sh -n 10000

plot:
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py; \
	)

test: 
	bash run.sh -n 1000
	mkdir -p ${PLOT_DIR}/test
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py test; \
	) 
	
