Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do
Runtime stuff:
* consider supporting a way to reconstruct the lexical environment at runtime
  * such that it would be possible to, e.g., have an `environment` function that returns the current lexical environment as a hashmap
  * the main purpose would be for interactive error handling
    * it would be nice if errors could be handled by basically giving you a repl wherever they occurred, but that isn't possible with the current evaluation model, since there isn't really an environment and instead values just exist in the registers
  * this seems like it would be pretty difficult
    * especially given that I plan to clear variables as soon as they are last used, instead of at the end of scope...
      * I guess this could be disabled in a debug mode?
        * I guess if I go this route I could also just not do register allocation in debug mode and make everything use the environment? That would make implementing this very easy, but would require pretty different compilation models
          * I guess if there's just one pass that that replaces all `Bind` instructions with register allocations then they wouldn't need to be all that different?
            * but functions are already implemented without using `Bind`...
* handle external errors
* destructuring
* support multi-arity composite functions
  * I guess this could be a vec of `(AritySpecifier, CompositeFunction)`, where `AritySpecifier` can describe a fixed num, a fixed range, or a n-to-infinity range
* replace coroutines with effect handlers
  * get rid of cells I guess? You can just emulate them with a "state" handler
* support laziness
  * add a new type for a lazy sequence (not sure what to call it... `Lazy`? `Iterator`?)
    * this should consist of a vec of realized values and a "realizer" (a rust iterator?) that can be used to generate the rest of the values
      * a rust iterator would work for the realizers of built-in functions, but I want there to be a function to turn a coroutine into a lazy list, and I'm not sure a rust iter could capture that...
        * I guess there could be a `Realizer` enum type with `Iterator` and `Coroutine` variants?
* support partially-applied functions
  * these should store a function and a vec of arguments passed to it
  * this will of course be helpful for implementing the `Partial` instruction, but also I think it will be necessary for lambda lifting
  * I guess the `Compose` and `Memoize` instructions might need special vm-level machinery too?
* support cells
* write tests that make sure the single-ownership `Rc` optimization is properly avoiding unnecessary clones
  * not sure exactly how to do this...`SingleArityCompositeFunction`
* implement remaining instructions, and write tests
* add an ability to overload certain core functions like `=` and `+` for specific `ExternalObject` types
  * for `=`, for example, this would work by having a function like `EvaluationState::add_external_eq_type<T: PartialEq>` that adds the `TypeId` of the provided type to a `HashMap` mapping to a function that uses the type's `PartialEq` to do the comparison
    * the same approach should work for pretty much anything else you'd want to overload, e.g. `Add` for `+`, `IntoIterator + FromIterator` to be callable with `map` and `filter`, etc.
    * this function won't need to take any arguments, as the function definition is the same for every type
    * example here: https://www.reddit.com/r/rust/comments/1ckgqrg/comment/l2nh7w5/
* implement core fns
* think about how to support parallelism
  * I guess it might be simple to just piggyback on rusts threads... maybe passing a coroutine or function into a `Spawn` instruction and having a `Await` instruction to join?
  * I don't wanna have to use `Arc`s everywhere...
    * might be time to implement my own `Rc` replacement at this point
  * clojure's channels API seems pretty nice, maybe just try to copy that
    * go blocks seem like maybe the tough part

Compiler stuff
* do real error handling for `allocate_registers` rather than `expect`ing everywhere
* support compiling list functions
  * `list`, `first`, `rest`, `last`, `butlast`, `push`
* support compiling functions
* support closures
  * will have to lambda lift them, this will probably be kinda tricky
* get rid of `EvaluationState::consumption`, determine stack frame offsets via results of lifetime analysis
  * rerun the benchmark in `main.rs` after this, curious how much of a difference it makes
* IR-level optimizations:
  * When a value is going to be passed into a `Call` at the end of its lifetime, use `StealArgument` rather than `CopyArgument`
  * [`Call`, `Return`] -> `CallAndReturn`
  * [`CallSelf`, `Return`] -> `CallSelfAndReturn`
  * [`Apply`, `Return`] -> `ApplyAndReturn`
  * [`ApplySelf`, `Return`] -> `CallSelfAndReturn`
  * Function inlining
  * Functions ending with `CallSelfAndReturn` that can put their return values back into the corresponding input registers can just `Jump` back to the start of the function
  * When a value reaches the end of its lifetime without being replaced, insert a `Clear` instruction
    * remember to shift the `RegisterLifetime` values around when doing this, timestamps beyond the point where this happens need to be incremented to account for the new instruction
    * This will reduce the reference count of shared collections, potentially avoid future cloning by allowing the collections to be mutated in place when used elsewhere
      * However, for non-collection values (just `Nil`, `Bool`, `Char`, and `Num`, I guess) this will actually make things slower, as it involves processing an extra instruction and zeroing out memory for no good reason
        * For this reason I'm unsure if this optimization will really be worth it. Should at least implement it and have it be an optional thing, and do some benchmarking to see what effect it has.
        * It should be possible to have certain instructions that are known to always produce non-collection values (arithmetic ops, boolean ops, etc...) tag their output registers with metadata that lets the compiler know it can skip the `Clear` at the end of the lifetime
          * but many things will often produce non-collection values, e.g. `First` and user-defined functions, in a way that the compiler can't know about because the language doesn't have static typing. So this will probably only help in a small fraction of cases
* Use GSE for parsing, once it's ready

# planned language features (once the IR and VM are usable and a basic AST)
* global defs
  * these will need special logic since I'm got rid of the `Bind` instruction, but that's fine since they're only allowed at the top level
* destructuring function arguments
* let blocks that desugar into a function calls
* match blocks
  * I guess these can desugar into a sequence of attempted function calls wrapped in `try` blocks or something? That way it could just defer to the normal function destructuring logic for matching...
    * except that wouldn't handle matches where certain things are fixed to be specific literal values...
    * also relying on the error handling system here might be bad for performance
* shadowing will not be allowed by default, but there will be a special kind of `let` that allows shadowing, maybe `let-shadow` or just `shadow` or something
  * so many times in clojure I've accidentally shadowed the `str` function with a local variable and wasted a lot of time trying to figure out why stuff wasn't working. Having the compiler refuse to let you shadow with explicitly stating that you know that's what you're doing would be really nice.
    * I considered just not allowing shadowing at all, but I think for some metaprogramming purposes it might be useful, so allowing it only in a special form seems nice
* quasiquoting
* macros
* top-level unquoting

# Long-run optimizations
* using the `take_mut` crate can probably avoid replacing a stolen register with a temporary `Nil` for several instructions
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
  * A reference-counted-vector type that only involves one layer of indirection rather than the 2 of an `Rc<Vec<>>` would also be nice for reprenting functions 
* Distinguish between multi-use constants and single-use constants
  * if a constant is known to be single-use, it could just be swapped with `Nil` in the constants stack rather than needing to be cloned when it's used
  * this could be especially important for functions with many instructions inside
  * this will require some static analysis, but probably nothing to crazy
    * will need to be recursive to fully catch all places where this optimization could be done, e.g. a constant that's used only once inside a function that's used only once an be made single-use, but you'd need to identify the function as single-use first to notice that
  * if the same constant is used multiple times in different places, it would actually be best to *not* make that a single constant that's used multiple times but instead a bunch of different copies of the same constant, such that each of them can be taken as single-use
  * anything declared globally should be inelligible for being declared single-use, since it could always be used an indeterminite number of times in the future by the repl
  * I guess rather than technically having a different kind of constant, which I guess would mean having a separate `single_use_constants` field in `Program`, there could just be a `StealConst` instruction in addition to the normal `Const` function
* Have special types for lists (and hashmaps and hashsets, presumably) that aren't stored in `Rc`s and are just directly mutable, for values that can be determined at compile-time to be single-use.
  * This should save some overhead for calling `get_mut` on the `Rc`s and deallocating the extra data associated with an `Rc`. This would also remove some indirection, if I can't figure out how to make the `RcVec` thing work
* Consider implementing specialized subtypes of `List` that handle certain operations in a more efficient way.
  * For instance, there could be a `SubList` that consists of an `Rc` to a normal list along with a start and end index, and operations like `get` or `count` could have special implementations for these types
    * Things like this could help avoid cloning data in a lot of cases
    * Other potential specialized types:
      * `ConcatList` - consists of two `Rc`s to normal lists, or maybe a `MiniVec<Rc>`
      * `PushedList` - an `Rc` to an existing (shared) list and an owned list that can be pushed to, allowing single-ownership-like mutation speeds despite the early part of the list being shared
      * `ConsedList` - an `Rc` to an existing (shared) list and a *backwards* single-ownership list, such that `Cons` instructions can just push onto the end of the secondary list
* partial evaluation on the bytecode
