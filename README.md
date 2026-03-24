# static-load
A thin wrapper over hazard pointers to facilitate reloadable global state. 

Unlike framework-level state (i.e axum), static-loaded resources cross 'static boundaries (i.e. tokio tasks)

Unlike Mutex or RwLock, static-loaded resources are never contended. Reading a resource is usually so fast
it can be considered free, like dereferencing a pointer.

# todo
- [ ] watching resource changes (n0-watcher)
- [ ] common boilerplate for tokio+tracing (optional)
- [ ] benchmarks