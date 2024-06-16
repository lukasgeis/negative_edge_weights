.PHONY: all run plot test clean prep

all: ./*/
	@for f in $^; do  \
		make all -C "$$f";\
	done

run: ./*/
	@for f in $^; do  \
		make run -C "$$f";\
	done

plot: ./*/
	@for f in $^; do  \
		make plot -C "$$f";\
	done

test: ./*/
	@for f in $^; do  \
		make test -C "$$f";\
	done

clean: ./*/
	@for f in $^; do  \
		make clean -C "$$f";\
	done

prep:
	( \
		python3 -m venv ../pyenv; \
		. ./../pyenv/bin/activate; \
		pip3 install matplotlib pandas seaborn; \
	)
