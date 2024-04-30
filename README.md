Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do
VM stuff:
* lists (secretly vectors, but I'm gunna call them lists to make Lisp people mad >:D)
  * just use vecs for now and do copy-on-write, replace it with my persistent vector once that's finished
* multi-argument functions
* strings
  * maybe implement as `Rc<str>` rather than `String`?
* hashmaps
* figure out what to do about laziness...
  * unsure of how to represent this.
    * Should I go for the same approach as Quoot?
      * i.e. have a `LazyList` type that consists of:
        * a vec of current values
        * a function that accepts the vec of current values and the index to produce the next value
      * this approach felt pretty messy
    * Maybe I could have an `Iter` type that mostly just wraps rust's iterator system? Though it would probably still need to be composed of both a `vec` of already realized values and an `iter` that can produce the rest of the values
      * typing here might get tricky, probably would have to use `dyn Iter`, though the other approach would also need something like this
* how is `apply` going to work on built-in operations? I guess for each operation there needs a corresponding function that does runtime arity checking and dispatch?
  * This makes sense I guess. This means that there can be slightly different logic for the `apply`d version, which can be good in some cases like `(apply + x)`, where the dynamically dispatched function version of `+` can do a rust-level reduce to sum all the elements of `x`, while expressions that use `+` normally can just be compiled into a bunch of binary `Add` instructions.

Language stuff:
* Finish GSE (in its repo)
* Specify a default parser
* Start on a compiler
  * start with just arithmetic stuff
  * lists/vectors
    * can just start with the basic operations for now
      * push, pop, count, concat
  * functions
    * wanna use CPS and ANF
    * not entirely sure how stack tracing works in register-based VMs
      * is it even strictly necessary? Definitely want to be able to do it for debugging purposes, but it feels like if the compiler keeps good track of the active registers then it feels like you could just do a function application without needing to do the equivalent of pushing a stack frame
  * let bindings
    * should this just be a macro that expands to a function call?? this is how CL does afaik. I feel uncertain about whether this makes sense from a performance perspective, not sure tho. Maybe CL can get away with it because it's compiler is very advanced??

# Long-run optimizations
* Reimplement `Value::List` using a custom reference-counted vector.
  * This could take more advantage of the fact that the behavior is very different when the reference count is 1. Also, `Rc<Vec<Value>>` involves two layers of indirection, but it should be possible to implement a custom `RcVec` with just one. This might be something like an enum with two variants for the single-ownership (mutable) case and the shared-ownership (immutable) case, like:
    ```rust
    enum RcVec<T> {
      Unique(MiniVec<T>)
      Shared(Box<{
        reference_count: usize,
        data: [T]
      }>)
    }
    ```
  * This layout would mean that changing between a `Unique` and a `Shared` would involve cloning the `MiniVec` data to make it into a `[T]` slice or vice versa tho... not ideal. It needs to be very cheap to change between the two or the performance gains from the lack of extra indirection might not be worth it. I guess the memory layout could be more like just a `MiniVec` but with an extra `reference_count` value in the vec header, so the layout would look like:
    ```rust
    (reference_count: usize, len: usize, cap: usize, data: [T; cap])
    ```
  * But that would mean that even checking whether the reference count is 1, i.e. whether it can be mutated, would involve a heap lookup. I guess there could be an extra bool stored on the stack that keeps track of whether the reference count on the heap is 1 or not... So the full memory layout would look something like:
    ```rust
    struct RcVec<T> {
      unique: bool,
      data: Box<{
        reference_count: usize,
        len: usize,
        cap: usize,
        data: [T; cap]
      }>
    }
    ```
  * This should fit in just 9 bytes, so it wouldn't make `Value` any bigger. Getting all the implementation details right to be as well-optimized as `Vec` or `MiniVec` might be pretty difficult tho.
