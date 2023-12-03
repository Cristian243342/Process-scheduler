.PHONY: outputs round-robin

export TREM := xterm

define banner
	@printf "\n"
	@printf "$$(TERM=xterm tput bold)********************************************************************************$$(TERM=xterm tput sgr0)\n"
	@string="$(1)" && printf "$$(TERM=xterm tput bold)* %-$$((76))s *\n" "$$string"
	@printf "$$(TERM=xterm tput bold)********************************************************************************$$(TERM=xterm tput sgr0)\n"
	@printf "\n"
endef

install:
	rm -rf ../Makefile
	cp Makefile ..
	rm -rf ../outputs
	cp -r outputs ..
	rm -rf ../runner/src/tests.rs
	rm -rf ../runner/src/tests
	cp -r tests ../runner/src/tests

outputs:
	rm -rf outputs
	# round robin
	WRITE_OUTPUT=true timeout 10 cargo test --bin "runner" --features="round-robin"
	WRITE_OUTPUT=true TIMESLICE=5 REMAINING=2 timeout 10 cargo test --bin "runner" --features="round-robin"
	WRITE_OUTPUT=true TIMESLICE=3 REMAINING=3 timeout 10 cargo test --bin "runner" --features="round-robin"

	# priority queue
	WRITE_OUTPUT=true timeout 10 cargo test --bin "runner" --features="priority-queue"
	WRITE_OUTPUT=true TIMESLICE=5 REMAINING=2 timeout 10 cargo test --bin "runner" --features="priority-queue"
	WRITE_OUTPUT=true TIMESLICE=3 REMAINING=3 timeout 10 cargo test --bin "runner" --features="priority-queue"

	# cfs
	WRITE_OUTPUT=true timeout 10 cargo test --bin "runner" --features="cfs"
	WRITE_OUTPUT=true CPU_SLICES=12 REMAINING=2 timeout 10 cargo test --bin "runner" --features="cfs" 
	WRITE_OUTPUT=true CPU_SLICES=18 REMAINING=3 timeout 10 cargo test --bin "runner" --features="cfs"

round-robin:
ifndef TEST
	$(error No test defined)
endif
	$(call banner,Round Robin Timeslice: 3 Remaining: 1)
	timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Round Robin Timeslice: 5 Remaining: 2)
	TIMESLICE=5 REMAINING=2 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Round Robin Timeslice: 3 Remaining: 3)
	TIMESLICE=3 REMAINING=3 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"

priority-queue:
ifndef TEST
	$(error No test defined)
endif
	$(call banner,Priority Queue Timeslice: 3 Remaining: 1)
	timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Priority Queue Timeslice: 5 Remaining: 2)
	TIMESLICE=5 REMAINING=2 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Priority Queue Timeslice: 3 Remaining: 3)
	TIMESLICE=3 REMAINING=3 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"

cfs:
ifndef TEST
	$(error No test defined)
endif
	$(call banner,Completely Fair Scheduler CPU Slices 10 Remaining: 1)
	timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Completely Fair Scheduler CPU Slices 12 Remaining: 2)
	CPU_SLICES=12 REMAINING=2 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
	$(call banner,Completely Fair Scheduler CPU Slices 18 Remaining: 3)
	CPU_SLICES=18 REMAINING=3 timeout 10 cargo test --bin "runner" $(TEST) -q --features="$@"
