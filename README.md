# `gitstat` — OOP Reference Guide (Completed Sections)

This is a standalone reference for anyone picking up the `gitstat` project.
It covers everything done so far — **`git.rs`** and **`discovery.rs`** —
explained through an Object-Oriented Programming lens for readers coming
from Java, Python, C#, etc. Rust doesn't have classes or inheritance, but
every classic OOP idea has a direct Rust equivalent, and this doc maps them
explicitly.

---

## 1. The OOP-to-Rust Translation Table

Keep this table open while reading the code sections below — every callout
refers back to a row in it.

| OOP concept | Traditional OOP | Rust equivalent | Where it shows up |
|---|---|---|---|
| Class | Bundles state + methods in one unit | `struct` (state) + `impl` block (methods), often physically separate | `RepoStats` struct in `git.rs` |
| Interface / abstract class | Declares a method contract without implementation | `trait` | `Analyzer` trait (covered in a future session) |
| Inheritance | Child class reuses/extends parent's implementation | **Does not exist.** Rust uses composition and trait implementation instead | N/A — see below |
| Polymorphism | One interface, many implementations, resolved at runtime | `Box<dyn Trait>` (runtime) or generics (`fn f<T: Trait>`, compile-time) | Planned for `Analyzer` |
| Exception hierarchy | `class MyError extends Exception` chains | One `enum` with variants, `#[derive(Error)]` via `thiserror`, matched exhaustively with `match` | `GitStatError` (used in `git.rs`) |
| Constructor | `new MyClass(...)` | Plain struct literal `MyStruct { field: val }`, or conventional `fn new(...) -> Self` | Not yet used, but this is the pattern to expect |
| Encapsulation | `private` fields + getters/setters | Module boundaries (`pub`/private by default) + struct field visibility | Each `.rs` file is its own encapsulation boundary |
| Method overriding | Subclass replaces parent's method body | Each trait implementor writes its own method body — no "parent" body exists to override | Will apply once `Analyzer` has multiple implementors |

**The single biggest mental shift:** in classic OOP, polymorphism usually
comes from an inheritance tree (`Animal → Dog`, `Dog` overrides
`makeSound()`). In Rust, there is no tree. Every type stands alone; it
*opts in* to behaving like an interface by implementing a `trait` for
itself. Two types that implement the same trait share **zero** code unless
the trait itself provides a default method body — there's no parent to
inherit from.

---

## 2. Encapsulation at the File/Module Level

Before the class-level detail, it's worth naming the encapsulation strategy
of the whole project, because it's very "OOP single-responsibility
principle" even without classes:

- **`git.rs`** owns *talking to git*. Nothing outside this file should know
  or care that a `Command::new("git")` call is happening under the hood.
- **`discovery.rs`** owns *finding repos*. It doesn't know how to read a
  repo once found — that's `git.rs`'s job.

This is the Rust-module version of "each class has one reason to change."
If shelling out to `git` is later swapped for the `git2` library
(libgit2 bindings — a stretch goal), only `git.rs` needs to change.
`discovery.rs`, `main.rs`, and everything downstream are unaffected because
they only ever see `git.rs`'s public interface (its `pub fn`s), never its
internals. That public/private line **is** encapsulation — it plays the
same role `private` fields + public methods play in Java or C#.

---

## 3. Deep Dive: `git.rs`

### What it does (the "class" summary)

Think of `git.rs` as a class whose only job is: *given a repo path, produce
a `RepoStats` object describing it.* If you were writing this in Java,
you'd sketch it as:

```java
class GitReader {
    RepoStats readStats(Path repoPath) throws GitStatException { ... }
}
```

In Rust, there's no `GitReader` object to instantiate — the equivalent is a
free function that takes the path and returns a `Result`:

```rust
pub fn read_stats(path: &Path) -> Result<RepoStats, GitStatError> { ... }
```

`RepoStats` is the "return type as data class" — analogous to a Java POJO
or a Python `@dataclass`. It holds the *result*, not any behavior.

### Key pieces, explained in OOP terms

**1. Shelling out to git**
```rust
Command::new("git").arg("-C").arg(path).arg("log").arg("--pretty=format:%an").output()
```
`Command` here is playing the role of a **builder object** — a very common
OOP pattern (think `StringBuilder` in Java, or a fluent API). Each
`.arg(...)` call returns `self` again, letting you chain configuration
calls before finally calling `.output()`, which is the "terminal method"
that actually executes and gives you a result. This is functionally
identical to something like:

```java
new ProcessBuilder("git", "-C", path, "log", "--pretty=format:%an").start();
```

**2. Error translation — `.map_err(...)?`**

This is the Rust version of a `try { ... } catch (SomeException e) { throw
new MyDomainException(e); }` block, but done inline and without exceptions.
`.map_err(...)` converts one error type into another (here, into
`GitStatError`), and the `?` operator is "return early if this was an
error" — the non-exception equivalent of an uncaught exception propagating
up the call stack. The difference from classic OOP exceptions: this
propagation is visible in the function signature (`-> Result<T,
GitStatError>`), not hidden — a caller *must* acknowledge the function can
fail, the compiler won't let them ignore it.

**3. The "did it actually work" check**
```rust
output.status.success()
```
This exists because `Command::output()` succeeding only means "the git
process was spawned and ran to completion" — not that git itself was
happy. It's the equivalent of checking an HTTP response's status code even
though the network request itself didn't throw. A path that isn't a repo
still "successfully" runs `git log`, it just exits with a non-zero status.

**4. The counting loop — encapsulated state mutation**
```rust
for line in stdout.lines() {
    let name = line.trim();
    if name.is_empty() { continue; }
    commit_count += 1;
    *authors.entry(name.to_string()).or_insert(0) += 1;
}
```
`authors` is a `HashMap<String, usize>` — the direct equivalent of a Java
`HashMap<String, Integer>` or Python `dict`. The OOP instinct here would be
to reach for `authors.containsKey(name) ? authors.get(name) + 1 :
authors.put(name, 1)` — two lookups, awkward. `.entry(name).or_insert(0)`
is Rust's built-in idiom for "get-or-create-then-mutate" in a *single*
lookup: "find this key; if it's missing, insert it with value `0`; either
way, hand me a mutable reference to the value so I can `+= 1` it."

### Errors as an enum, not an exception hierarchy

In Java you might have:
```java
class GitStatException extends Exception {}
class RepoNotFoundException extends GitStatException {}
class GitSpawnFailedException extends GitStatException {}
```
— a whole inheritance tree, one `class` per failure mode. `git.rs` instead
uses **one flat `enum`**:
```rust
#[derive(Error, Debug)]
enum GitStatError {
    SpawnFailed(std::io::Error),
    NotARepo(PathBuf),
    // ...
}
```
Every variant is a case of the *same* type, not a subclass. Handling it is
`match` (exhaustive — the compiler forces you to handle every variant, or
explicitly ignore the rest with `_`), not `catch` blocks. `#[derive(Error)]`
from the `thiserror` crate is what makes this enum behave like a proper
error type (implementing `std::error::Error`) without writing that
boilerplate by hand — comparable to how `extends Exception` gives you
stack traces and `Throwable` behavior for free in Java.

---

## 4. Deep Dive: `discovery.rs`

### What it does (the "class" summary)

If `git.rs` is "read one repo I already found," `discovery.rs` is "find
*where* the repos are in the first place." Its OOP sketch:

```java
class RepoFinder {
    List<Path> findRepos(Path root, int maxDepth) { ... }
}
```

Rust version — again, a free function instead of an object:
```rust
pub fn find_repos(root: &Path, max_depth: usize) -> Vec<PathBuf>
```

### Two implementations were compared — same "interface," different strategy

This is worth calling out in OOP terms because it's a textbook case of
**two different algorithms satisfying the same contract** — the same
signature, same inputs/outputs, different internal strategy. In classic
OOP you'd express "same contract, different strategy" via the **Strategy
pattern**: an interface with two interchangeable implementations. Here
there's no formal interface (it's just one function, replaced outright),
but the comparison is exactly a Strategy-pattern trade-off analysis:

| | Original version | User's version |
|---|---|---|
| Skips crawling into `.git` internals | ❌ | ✅ |
| Skips noise dirs (`node_modules`, `target`) | ✅ | ❌ |
| Handles `.git` as a file (worktrees/submodules) | ✅ | ❌ (`is_dir()` requires it to be a folder) |
| Has a `max_depth` limit | ✅ | ❌ (originally missing) |

**Original approach — check from outside:**
```rust
if entry.path().join(".git").exists() {
    repos.push(entry.path().to_path_buf());
}
```
For every directory the walker visits, ask "does this directory contain a
`.git`?" Simple, but wasteful: the walker still descends *into* `.git`
afterward and crawls git's internal object storage, since nothing tells it
to stop.

**User's approach — react from inside, then prune:**
```rust
let mut it = WalkDir::new(root).into_iter();

while let Some(entry) = it.next() {
    let entry = match entry {
        Ok(e) => e,
        Err(_) => continue,
    };

    if entry.file_type().is_dir() && entry.file_name() == ".git" {
        if let Some(parent) = entry.path().parent() {
            repos.push(parent.to_path_buf());
        }
        it.skip_current_dir();
    }
}
```
This waits until the walker has actually stepped *into* a `.git` folder,
records its **parent** as the repo root, then calls `it.skip_current_dir()`
— an explicit instruction to the iterator: "don't bother descending further
here." This is genuinely more efficient on repos with large histories,
because it prunes the walk instead of merely ignoring what it finds.

**Why a `while let` loop instead of `for`:** in OOP terms, `it.next()` is a
method call on an **iterator object** that has internal mutable state (its
current walk position). A `for` loop in Rust takes *ownership* of the
iterator and drives it opaquely — you never get to call other methods on it
mid-loop. `while let Some(entry) = it.next()` keeps `it` as a
regular variable you still hold a handle to, so you can call
`it.skip_current_dir()` on it between iterations. This is analogous to the
difference between a Java for-each loop (`for (x : list)`, no access to the
underlying `Iterator`) versus manually driving `Iterator<T> it =
list.iterator(); while (it.hasNext()) { ...; it.remove(); }` — you drop
down a level specifically to call an extra method the simple loop form
doesn't expose.

### Bug found and fixed

```rust
use walkdir::{Walkdir};   // ❌ wrong casing — module is WalkDir
use walkdir::WalkDir;     // ✅ fixed
```
Rust identifiers are case-sensitive, same as Java/C#/Python. This is the
equivalent of `import java.util.Arraylist;` when the actual class is
`ArrayList` — a one-character typo that's a hard compile error, not a
warning.

### Final merged version (bugfix + `max_depth` restored)

```rust
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_repos(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut it = WalkDir::new(root).max_depth(max_depth).into_iter();

    while let Some(entry) = it.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
            it.skip_current_dir();
        }
    }

    repos
}
```
Note the signature takes **two arguments** (`root`, `max_depth`) — `main.rs`
must call it as `find_repos(root, max_depth)` for this to compile.

**What's still missing from the user's version, if picking it back up:**
noise-directory filtering (`node_modules`, `target`, `.cargo`) and handling
`.git` as a *file* rather than a folder (needed for worktrees/submodules,
where `.git` is a one-line pointer file, not a directory) — both present in
the original version but not yet folded into this merged one.

---

## 5. Glossary Cheat-Sheet (quick lookup while reading Rust code)

| Rust term | Nearest OOP equivalent |
|---|---|
| `struct` | class's data/fields |
| `impl Foo { ... }` | class's methods, in a separate block |
| `trait` | `interface` |
| `impl Trait for Foo` | `class Foo implements Trait` |
| `Box<dyn Trait>` | a variable typed as the interface, holding any implementor (runtime dispatch) |
| `enum` + `match` | exception hierarchy + `catch`, or a sealed class hierarchy |
| `Result<T, E>` | checked exception, but enforced by the compiler instead of a `throws` clause |
| `Option<T>` | nullable reference, but the compiler forces a null-check before use |
| `?` operator | rethrow / propagate exception upward |
| module (`pub`/private) | package + access modifiers |
| `.entry(k).or_insert(v)` | `map.computeIfAbsent(k, v)` (Java) |

---

*Sections not yet covered (to be added in a follow-up doc): `analyzer.rs`
(the `Analyzer` trait), a second `Analyzer` implementation, `Box<dyn
Analyzer>` runtime polymorphism, and the `git2`-crate stretch goal.*