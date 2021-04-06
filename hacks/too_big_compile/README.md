```shell
$ cargo run -- -w hacks/too_big_compile/ -f 10240000 -- /usr/bin/g++ main.cpp -o a.out
g++: internal compiler error: File size limit exceeded signal terminated program as
Please submit a full bug report,
with preprocessed source if appropriate.
See <file:///usr/share/doc/gcc-9/README.Bugs> for instructions.
time_used = 14
memory_used = 18184
exit_code = 4
status = 1024
signal = 0
```