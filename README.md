Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do
Compiler/Runtime stuff:
* implement conditionals
* write a test for recursion
* start work on IR
  * This representation will simplify some things relative to the bytecode:
    * Constants could just be inlined into the IR values, there would be no need for a separate constant table at that level
      * could also get rid of the `program!` macro, or at least simplify it :P
    * The AST->IR compiler could use `usize` for registers and just use them in SSA form, and the IR->bytecode compiler could handle register reallocation. This would make lifetime analysis somewhat easier, and in some cases it might even be necessary - functions with >256 local variables might be very difficult/impossible to compile directly to the bytecode format.
      * Hmm, how would instructions like `If` that overwrite their arguments fit into SSA? Would the IR also represent them as overwriting their arguments or would it have them assign to a new register? I think the former might make it harder to make use of the nice properties that SSA provides, but the latter might make the IR->bytecode compilation process more complicated.
        * Actually I guess it wouldn't be much more complicated... once liftime analysis has been performed the IR->bytecode compiler should be able to easily tell whether the bytecode can reuse the register of the argument or whether the argument needs to be copied into a new register to avoid being overwritten.
    * The AST->IR compiler could just have one type of `Apply`, which the IR->bytecode compiler could then convert to the specialized `Apply<X>` or `Apply<X>AndReturn` instructions.
      * This would feel a bit more elegant but I'm not sure if it's a real advantage...
    * Instructions like `Add`, `Multiply`, `List` that take a variable number of arguments could be represented more elegantly in the IR.
      * would this be problematic for making use of the SSA form tho?
* Implement compiler from IR to bytecode
  * Compute lifetimes of all virtual registers, reallocate them into a smaller number of real registers for the bytecode
  * Calls to `Apply` will, depending on their arity, be converted into `Apply0`, `Apply1`, `Apply2`, or `ApplyN` instructions. In the `ApplyN` case, it will also need to emit `EmptyRawVec` and `CopyIntoRawVec`/`StealIntoRawVec` instructions to construct the argument vector.
  * Optimizations (not essential at first):
    * All occurrences of `Apply` followed by `Return` should be converted into `Apply<X>AndReturn` instructions rather than normal `Apply<X>` instructions
* write tests that make sure the single-ownership `Rc` optimization is properly avoiding unnecessary clones
  * not sure exactly how to do this...
* implement `Apply<X>AndReturn` instructions
  * implement tests for these based on equality checking between programs
* support multi-arity composite functions
  * probably have a new `MultiArityCompositeFunction` type, rename the current `CompositeFunction` type to `SingleArityCompositeFunction`
* figure out what to do about laziness...
  * unsure of how to represent this.
    * Should I go for the same approach as Quoot?
      * i.e. have a `LazyList` type that consists of:
        * a vec of current values
        * a function that accepts the vec of current values and the index to produce the next value
      * this approach felt pretty messy
    * Maybe I could have an `Iter` type that mostly just wraps rust's iterator system? Though it would probably still need to be composed of both a `vec` of already realized values and an `iter` that can produce the rest of the values
      * typing here might get tricky, probably would have to use `dyn Iter`, though the other approach would also need something like this
* support coroutines
* support custom instructions
  * the definitions of these should be stored in `EvaluationState`, such that you can create an `EvaluationState` and then add custom instructions to it before doing any evaluation
* support a `Foreign` type, a general-purpose boxed-rust-object type
* implement remaining instructions, and write tests
* start on a compiler from ASTs into IR
* Once GSE is ready, specify a basic grammer, and set up a function that accepts a GSE string, parses it, compiles it to the IR, compiles that to the bytecode, and then runs it.

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
* `EvaluationState::consumption` shouldn't *really* need to be tracked at runtime
  * Right now every single call to `set_register` has to update `consumption`, so getting rid of that might mean a fairly significant reduction in overhead
  * The only thing that it's actually used for at runtime (other than debugging) is to figure out where to start a new stack frame when a function is called, but that should be determinable at compile time via static analysis.
    * There would need to be a new special instruction like `SetStackFrameOffset(u8)` that the compiler places before calls to `Apply<X>`, which sets a global value that is then used in the `Apply<X>` instruction handler to determine where to start the new stack frame (relative to the old). This does mean there will be slightly more overhead in function dispatching, but I think it would definitely be worth it for the save in updating `consumption` with every instruction.
    * Doing the static analysis would just involve finding the highest register that has been touched
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
  * I guess this could just use `RawVec` in the case of lists?
* Consider implementing specialized subtypes of `List` that handle certain operations in a more efficient way.
  * For instance, there could be a `SubList` that consists of an `Rc` to a normal list along with a start and end index, and operations like `get` or `count` could have special implementations for these types
    * Things like this could help avoid cloning data in a lot of cases
    * Other potential specialized types:
      * `ConcatList` - consists of two `Rc`s to normal lists, or maybe a `MiniVec<Rc>`
      * `PushedList` - an `Rc` to an existing (shared) list and an owned list that can be pushed to, allowing single-ownership-like mutation speeds despite the early part of the list being shared
      * `ConsedList` - an `Rc` to an existing (shared) list and a *backwards* single-ownership list, such that `Cons` instructions can just push onto the end of the secondary list