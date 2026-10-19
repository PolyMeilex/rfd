# GTK threading manual tests

GTK backends are ridiculously complex, as GTK is a full blown GUI framework, not a dialog provider.  
So here is a set of weird manual test edge cases meant to check if this shit even works.

The quality of those tests is dubious, but it's better than nothing.

```sh
./run-all.sh gtk4
./run-all.sh gtk3
```

# Cases

| Binary                    | App uses GTK | Case                                                                                     |
| ------------------------- | ------------ | ---------------------------------------------------------------------------------------- |
| `main_sync`               | no           | Sync dialogs from the main thread                                                        |
| `main_async_block_on`     | no           | Async dialogs awaited on the main thread with `block_on`                                 |
| `async_awaited_elsewhere` | no           | Async future created on main thread, awaited on a worker thread                          |
| `worker_sync`             | no           | Sync dialogs from a worker thread                                                        |
| `parallel_sync`           | no           | 3 threads open sync dialogs at the same time, all 3 should be shown                      |
| `parallel_async`          | no           | 3 async dialogs at once, awaited with `join!`, all 3 should be shown                     |
| `sequential_mixed`        | no           | Sync and async dialogs from different threads, one after another                         |
| `dropped_future`          | no           | Async future dropped without awaiting, later dialogs should work                         |
| `exit_after_dialog`       | no           | Process exits right after a dialog returns, racy, run it in a loop                       |
| `gtk_callback_sync`       | yes          | Sync dialogs from a GTK callback, on the GTK thread (nested main loop)                   |
| `gtk_callback_async`      | yes          | Async dialogs awaited on the GTK thread itself, via `spawn_local`                        |
| `gtk_worker_sync`         | yes          | GTK loop on main thread, worker thread opens sync dialogs                                |
| `gtk_nested_sync`         | yes          | While a sync dialog is open from a GTK callback, a timer opens another sync dialog       |
| `gtk_init_c`              | yes          | GTK initialized through the C API (bypassing gtk-rs), dialogs from callback and worker   |
| `gtk_before_loop`         | yes          | Sync dialog after GTK init, but before the main loop starts, `--c` to init through C API |
| `gtk_off_main_thread`     | yes          | App runs GTK on a non-main thread, main thread opens sync and async dialogs              |
