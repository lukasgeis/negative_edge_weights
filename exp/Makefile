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

clean: ./*/
	@for f in $^; do  \
		make clean -C "$$f";\
	done
