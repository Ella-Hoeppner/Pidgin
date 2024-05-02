Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do
VM stuff:
* make `program!` macro capable of handling `Const` instructions inside composite functions
* implement `ApplyN`
  * this will need some supplementary instructions to pack the arguments into a vector...
* implement `Apply<X>AndReturn` instructions
* start work on an optimizer
  * for now this should just find occurrences of `Apply<X>` followed by `Return`, and convert them into `Apply<X>AndReturn`
  * implement tests for these based on equality checking between programs
* implement `CoreFn` support
* figure out what to do about laziness...
  * unsure of how to represent this.
    * Should I go for the same approach as Quoot?
      * i.e. have a `LazyList` type that consists of:
        * a vec of current values
        * a function that accepts the vec of current values and the index to produce the next value
      * this approach felt pretty messy
    * Maybe I could have an `Iter` type that mostly just wraps rust's iterator system? Though it would probably still need to be composed of both a `vec` of already realized values and an `iter` that can produce the rest of the values
      * typing here might get tricky, probably would have to use `dyn Iter`, though the other approach would also need something like this
* think about representing continuations
* implement remaining instructions and tests

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
* Distinguish between multi-use constants and single-use constants
  * if a constant is known to be single-use, it could just be swapped with `Nil` in the constants stack rather than needing to be cloned when it's used
  * this could be especially important for functions with many instructions inside
  * this will require some static analysis, but probably nothing to crazy
    * will need to be recursive to fully catch all places where this optimization could be done, e.g. a constant that's used only once inside a function that's used only once an be made single-use, but you'd need to identify the function as single-use first to notice that
  * if the same constant is used multiple times in different places, it would actually be best to *not* make that a single constant that's used multiple times but instead a bunch of different copies of the same constant, such that each of them can be taken as single-use
  * anything declared globally should be inelligible for being declared single-use, since it could always be used an indeterminite number of times in the future by the repl