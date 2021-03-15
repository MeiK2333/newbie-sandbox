# newbie-sandbox

菜鸡沙盒

## TODO

- 移除 Tokio，二进制文件无需异步支持
- runit
- wait
- cgroup v1
- cgroup v2
- seccomp

## 初始化

```bash
# 通过 docker 获取系统文件并复制到本地
./build.sh
```

## 启动 cgroup v2

如果系统已经是 cgroup v2 则忽略此步。

内核版本较老的（ < 4.15 ）请不要进行此操作，否则可能会造成系统异常

```bash
vim /etc/default/grub
# 将参数添加至 GRUB_CMDLINE_LINUX_DEFAULT="...... cgroup_no_v1=allow systemd.unified_cgroup_hierarchy=1" 以禁用 cgroup v1
update-grub
# 重启以使改动生效
reboot
```

## Usage

```bash
# 因为 namespaces 的限制，必须以 root 权限执行
cargo build && sudo ./target/debug/newbie-sandbox /bin/bash
# 进入沙盒后进行的操作将无法影响到外部操作系统
nobody@nb_sandbox:/$ echo Hello World!
Hello World!
```
