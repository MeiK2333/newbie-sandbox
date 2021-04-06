```shell
$ g++ hacks/reboot/main.cpp -o hacks/reboot/a.out
$ cargo run -- -w hacks/reboot/ -- ./a.out
time_used = 0
memory_used = 972
exit_code = 0
status = 31
signal = 31
```

`reboot` 会被沙盒阻止，signal 为 31。