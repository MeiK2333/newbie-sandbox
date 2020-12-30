# newbie-sandbox

菜鸡沙盒

## 初始化

```bash
# 通过 docker 获取系统文件并复制到本地
./build.sh
```

## Usage

```bash
# 因为 namespaces 的限制，必须以 root 权限执行
cargo build && sudo ./target/debug/newbie-sandbox /bin/bash
# 进入沙盒后进行的操作将无法影响到外部操作系统
nobody@nb_sandbox:/$ echo Hello World!
Hello World!
```

## TODO

- Cgroup
