# WIP Rust util that parses and provides types to represent Stormworks .mesh files.
This library is a rewrite of parts of [CodeLeopard's C# program](https://gitlab.com/CodeLeopard/StormworksData). Full credit, the core parsing logic in this crate is effectively a simple port.

The code to parse a file into our relatively direct in-memory representation is done. I believe error handling is also in a nice state. When we left off, we were developing different versions (blocking, async, bevy async) and making performance comparisons, not in a very clean way due to our rapid progress. Another big thing that should be done is providing a way to convert to a more commonly used in-memory representation of a 3d model. Probably as an impl into something Bevy uses.

This entire project was made for another project of ours, to control a vehicle in Stormworks from an external program that reflects the ingame world.

# Old notes about performance comparisons:
We tried a few different approaches to get fast file loading. The speed benchmark was loading/parsing all ~200 files each with `build_mesh`. All times given are from my computer (Ryzen 7 7800x3d + 4800MT/s RAM + KINGSTON SKC3000D2048G)

Tokio io + synchronous loading was slowest
Std io + synchronous loading was decently faster (~680ms)
Tokio io + asynchronous loading was faster even still (~190ms)
Std io + asynchronous loading was the absolute fastest (~130ms)

When I write 'tokio io' I mean that the io, bufwriter, read_exact, etc stuff that tokio provides was used, and that all the lib functions were async.
When I write 'std io' I mean that the std io, bufwriter, read_exact, etc stuff was used, and that no lib function was async.
When I write 'synchronous loading', I mean that everything was loaded with a for loop over all paths to files to load.
When I write 'asynchronous loading', I mean that there was a `Vec<JoinHandle<Result<Mesh, String>>>` which was executed concurrently by `future::join_all`

For asynchronous loading, it was found that letting tokio go hog wild and try to load all ~200 files at the same time was not fast, but if semaphores were used (15 permits was experimentally found to be fastest), very quick loading could be achieved.

To view the benchmark code:
- Navigate somewhere to put the project.
- `git clone https://github.com/JudgementalBird/stormworksmeshutils`
- `git reset --hard 89a6e7bea0419a9176154e65ff8c78160db23a0c` (to go to a relevant commit with tests)
- Open in editor
Keep in mind the testing was done with tweaks being done back and forth, I don't think all tests mentioned are contained at that git commit, but I'm fairly sure that is the latest version with the tests included, and on it you can test 2 or 3 of the tests listed higher up in this message.


Up next is trying out https://docs.rs/bevy/latest/bevy/tasks/struct.AsyncComputeTaskPool.html