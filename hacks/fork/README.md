```shell
$ g++ hacks/fork/main.cpp -o hacks/fork/a.out
$ cargo run -- -w hacks/fork/ -c 2 -p 3 -- ./a.out
pid = 4
pid = 5
pid = 6
pid = -1
pid = -1
pid = -1
pid = -1
pid = -1
pid = -1
pid = -1
time_used = 1
memory_used = 1408
exit_code = 0
status = 0
signal = 0
```
