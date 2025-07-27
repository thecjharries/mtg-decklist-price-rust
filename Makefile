CARGO ?= cargo

.PHONY: test
test:
	$(CARGO) test -- --test-threads 1

.PHONY: coverage
coverage:
	$(CARGO) tarpaulin -v --fail-under=100
