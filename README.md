# static-reload
A thin wrapper over hazard pointers to facilitate reloadable global state. 

Unlike framework-level state (i.e axum), static-loaded resources cross 'static boundaries (i.e. tokio tasks)

Unlike Mutex or RwLock, static-loaded resources are never contended. Reading a resource is usually so fast
it can be considered free.

# benchmarks
Results from my Intel i7-8700

```
Timer precision: 100 ns
load                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ load_reloaded_resource  51.33 ns      │ 120.8 ns      │ 51.72 ns      │ 59.34 ns      │ 100     │ 25600
╰─ load_resource           31.8 ns       │ 157.5 ns      │ 31.8 ns       │ 34.63 ns      │ 100     │ 25600
```

As always, take micro-benchmarks with a grain of salt, as they do not represent a real workload.

# todo
- [ ] watching resource changes (n0-watcher)
- [x] common boilerplate for tokio
- [x] benchmarks