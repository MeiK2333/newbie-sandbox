```shell
$ g++ hacks/too_big_output/main.cpp -o hacks/too_big_output/a.out --static
$ cargo run -- -t 10000 -w hacks/too_big_output/ -o hacks/too_big_output/output.txt -f 10240 -- a.out
time_used = 0
memory_used = 740
exit_code = 0
status = 25
signal = 25
$ wc -m hacks/too_big_output/output.txt 
10240 hacks/too_big_output/output.txt
```