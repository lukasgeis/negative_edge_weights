DATA_DIR := ../../data/acceptance
PLOT_DIR := res

__data_dir := $(shell mkdir -p ${DATA_DIR})
__plot_dir := $(shell mkdir -p ${PLOT_DIR})

.PHONY: run plot test

run:
	bash run.sh -n 10000 -r 200

plot:
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py; \
	)

test:
	bash run.sh -n 100 -r 1
	mkdir -p ${PLOT_DIR}/test
	( \
		. ./../../pyenv/bin/activate; \
		python3 plot.py test; \
	)
