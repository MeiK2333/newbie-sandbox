```shell
$ g++ hacks/write_file/main.cpp -o hacks/write_file/a.out --static
$ cargo run -- -w hacks/write_file/ -- ./a.out output.txt
time_used = 0
memory_used = 808
exit_code = 0
status = 0
signal = 0
$ cat hacks/write_file/output.txt 
Hello World!
# 无法在当前目录外写入
$ cargo run -- -w hacks/write_file/ -- ./a.out /root/output.txt
time_used = 0
memory_used = 800
exit_code = 0
status = 11
signal = 11
$ cat rootfs/root/output.txt
cat: rootfs/root/output.txt: No such file or directory
```
