```shell
$ cargo run -- -w hacks/block_compile/ -t 10000 -- /usr/bin/g++ main.cpp -o a.out
time_used = 57
memory_used = 21680
exit_code = 0
status = 0
signal = 0
```

因为沙盒内部并没有阻塞的 `/dev/console` 与 `/dev/tty`，因此此攻击方式无效。
