CARGO ?= cargo

.PHONY: test
test:
	$(CARGO) test -- --test-threads 1
