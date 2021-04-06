```shell
$ g++ hacks/hello_world/main.cpp -o hacks/hello_world/a.out --static
$ cargo run -- -w hacks/hello_world/ -o hacks/hello_world/output.txt  -- a.out
time_used = 0
memory_used = 1636
exit_code = 0
status = 0
signal = 0
$ cat hacks/hello_world/output.txt 
Hello World!
```