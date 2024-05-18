Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do
General:
* destructuring
* Add a "debug mode" to the runtime + compiler. In this mode, each stackframe carries a HashMap describing the local environment. On the runtime side, this means there needs to be an extra `LocalBind` instruction that adds values to that hashmap. On the compiler side, that instruction needs to be emitted at the start of every function for each input, with the proper name given the lexical scope.
  * Most optimizations will probably have to be disabled in this mode. Specifically, function inlining will probably be impossible.
  * This mode will probably be *much* slower, not only because of the disabling of optimziations, but also because the local environment will keep the reference count of all collections above 1 at all times, so the runtime will have to clone a collection for every single modification
  * This will be helpful for debugging things tho, as it will be possible to write a error effect handler that e.g. prints the entire lexical scope to the console, or starts up a repl within that scope

Runtime stuff:
* for the `Const` instruction, take arguments out of the constants table rather than cloning them. I guess use `.replace(Nil)`. This will mean that often times the constants table will have to be cloned when entering a stack frame, but that can be avoided when the reference count of the block is 1 (not sure that it's easy to condition things on the reference count like that with the way things are currently set up, but this should be possible with some refactoring)
  * This does mean that constants in the table can never be shared, but that seems worth it. If they could be shared then it would be a lot more work to keep track of when they can and can't be tsolen from the table. Single use of constants make various things in compilation and IR transformation easier anyhow.
* support multi-arity composite functions
  * I guess this could be a vec of `(AritySpecifier, CompositeFunction)`, where `AritySpecifier` can describe a fixed num, a fixed range, or a n-to-infinity range
* make coroutines immutable
  * When invoked, they shouldn't modify the continuation referenced by the original coroutine object. Instead, they should clone that continuation, and only modify the clone. When a corutine yields, the it will always return a list of 2 values to the site that called it:
    1. a new coroutine, with a continuation from where the yield happened
    2. the value passed to the `yield` from inside the coroutine
  * This way, coroutines will be immutable objects, and if you want to continue from the place where they left off, you'll just use the new coroutine returned from the invocation of the last one. This will also mean that coroutines are multi-shot.
  * This can use the "steal rather than clone when RC=1" optimization on the continuations to avoid a big slowdown from unnecessarily cloning coroutines that are used as single-shot, while still fully supporting multi-shot coroutines
    * oof, I think this means that `Call` needs to be refactored to use a replacable register, used for both the thing being called and the return value...
      * that'll touch a lot of stuff... definitely doable though
      * I guess this same change will be necessary to do the "steal rather than clone when RC=1" optimization on the constant tables of function blocks, so this is worth doing anyways
* add effect handlers
  * get rid of cells I guess? You can just emulate them with a "state" handler
    * A state handler probably wouldn't be quite as efficient as cells... but on the other hand, this would make the language purely functional. Seems worth it I think?
    * I could keep cells but also have a "state" handler (or let people build their own), and encourage the use of the state handler by default, but have cells for performance-critical stuff?
      * Cells wouldn't really play nicely with mutliple resumptions of coroutines or continuations in effects, which could be a pretty big footgun.
      * this isn't really a performance-critical language of course, but the idea of taking out a feature that would help with performance just to avoid making people who aren't careful or don't know what their doing confused doesn't feel right...
        * maybe having the language be purely functional, i.e. not having atoms, would allow some optimizations later on that would outweigh this cost?
          * idk what exactly these optimizations would be though...
          * and even if the core language is purely functional, there's never really gunna be a way to guarantee that that still holds once rust interop is involved, so maybe that already rules out any hypothetical optimizations like that
      * Maybe if I optimize continuations/effects enough then there wouldn't even be a performance difference. Specifically I'm thinking of like - in situations where the compiler can guarantee that default "state" handler hasn't been overwritten, and the current scope isn't inside anything that could potentially be executed multiple times as a continuation, it could optimize away the state handler and just use actual mutable state?
* handle external errors
* support laziness
  * add a new type for a lazy sequence (not sure what to call it... `Lazy`? `Iterator`?)
    * this should consist of a vec of realized values and a "realizer" (a rust iterator?) that can be used to generate the rest of the values
      * a rust iterator would work for the realizers of built-in functions, but I want there to be a function to turn a coroutine into a lazy list, and I'm not sure a rust iter could capture that...
        * I guess there could be a `Realizer` enum type with `Iterator` and `Coroutine` variants?
* support partially-applied functions
  * these should store a function and a vec of arguments passed to it
  * this will of course be helpful for implementing the `Partial` instruction, but also I think it will be necessary for lambda lifting
  * I guess the `Compose` and `Memoize` instructions might need special vm-level machinery too?
* add string manipulation instructions
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
* support compiling the rest of the math functions
  * ==, zero?, nan?, even?, odd?, pos?, neg?, inc, dec, single-arg -, abs, floor, ceil, sqrt, exp, exp2, ln, log2, pow, mod, quot, min, max, >, <, >=, <=, rand
* support compiling boolean functions
  * =, not=, not, and, or, xor
* support compiling type checkers, converters
  * nil?, bool?, char?, num?, int?, float?, symbol?, str?, list?, map?, set?, collection?, fn?, error?, bool, char, num, int, float, symbol, to-list, to-map, to-set, error
* lambda-lifting
  * this will be an AST-level transformation
* support quoting
  * `Expression` should have a special variant case for this
    * lift_lambdas needs to avoid replacing quoted forms
* support compiling if statements
  * Maybe it would make sense have a new `EagerIf` instruction that takes 2 registers and just returns the value of one or the other based on a boolean register (which will also just be used as the return register). This wouldn't do a short-circuiting optimization. The compiler could then just emit, for an `(if ...)` statement, 2 thunks (which could later be lambda-lifted) for each side of the if, and use `EagerIf` to put one of the two into a register, then `Call` that function.
    * There could be an optimization pass that transforms this pattern into the more optimized `If` `Else` `End` instructions, but that pass can come relatively late in the compilation pipeline, so other passes don't have to worry about being the existence of `If`s or `Jump`s
      * However, this pass does need to be followed by a function inlining pass, as it would just intersperse `Call`s of the thunks between the `If` `Else` and `End` instructions, so it wouldn't be optimally efficient without a later function inlining step.
    * I guess this needs to happen at the AST level, since it will come before lambda-lifting. I guess `if` could just be a macro that transforms `(if <cond> <a> <B>)` into `((if-eager <cond> (fn () <a>) (fn () <b>)))`.
* IR-level optimization passess:
  * When a value is going to be passed into a `Call` at the end of its lifetime, use `StealArgument` rather than `CopyArgument`
  * [`Call`, `Return`] -> `CallAndReturn`
  * [`CallSelf`, `Return`] -> `CallSelfAndReturn`
  * [`Apply`, `Return`] -> `ApplyAndReturn`
  * [`ApplySelf`, `Return`] -> `CallSelfAndReturn`
  * [`Clear`, `Clear`] -> `Clear2`, [`Clear`, `Clear`, `Clear`] -> `Clear3`
    * need to implement `Clear2` and `Clear3` first
  * `Const(x, List(vec![]))` -> `EmptyList`
  * `Const(x, Int(i))` -> `ConstInt(x, i)` (when the int fits in an `i16`)
    * `ConstInt` will be a new instruction that holds a register index and an `i16`. Small integers come up a lot in practice, and this would avoid the indirection and overhead involved in looking into the constant table, so this seems pretty worthwhile.
  * [`EmptyList`, `Const`, `Push` ... `Const`, `Push`] -> [`Const(Full List)`]
    * Maybe even - if some elements are constant and some aren't, replace the `EmptyList .. Push` chain with a `Const(List)` with the size of the full list and all the constant values inlined, followed by `Set` instructions to add in the non-constant values
  * translate `Map` over constant lists can to a sequence of individual function applications
  * Function inlining
  * Functions ending with `CallSelfAndReturn` that can put their return values back into the corresponding input registers can just `Jump` back to the start of the function
  * When a value reaches the end of its lifetime without being replaced, insert a `Clear` instruction
    * remember to shift the `RegisterLifetime` values around when doing this, timestamps beyond the point where this happens need to be incremented to account for the new instruction
    * This will reduce the reference count of shared collections, potentially avoid future cloning by allowing the collections to be mutated in place when used elsewhere
      * However, for non-collection values (just `Nil`, `Bool`, `Char`, and `Num`, I guess) this will actually make things slower, as it involves processing an extra instruction and zeroing out memory for no good reason
        * For this reason I'm unsure if this optimization will really be worth it. Should at least implement it and have it be an optional thing, and do some benchmarking to see what effect it has.
        * It should be possible to have certain instructions that are known to always produce non-collection values (arithmetic ops, boolean ops, etc...) tag their output registers with metadata that lets the compiler know it can skip the `Clear` at the end of the lifetime
          * but many things will often produce non-collection values, e.g. `First` and user-defined functions, in a way that the compiler can't know about because the language doesn't have static typing. So this will probably only help in a small fraction of cases
      * Alternatively, what about making `Clear` not actually zero out the memory, but just like, decrement the strong count of the associated `Rc` (if any, i.e. if the value is a collection) and then use `std::mem::forget`? This would avoid zeroing out the memory. Though it would still mean processing an extra instruction.
* do real error handling for `allocate_registers` rather than `expect`ing everywhere
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
  * This could have a few advantages:
    * there would only need to be one layer of indirection in accessing the contents of the vector, rather than 2
    * it could switch from acting like a vector to acting like a deque without any reallocation, so pushing onto both the front and the back could be relatively cheap
  * representation could be something like:
    ```rust
    struct RcVec<T> {
      unique: bool,
      data: *const {
        reference_count: usize,
        len: usize,
        cap: usize,
        first_index: usize,
        data: [T; cap]
      }
    }
    ```
    * the `unique` field is redundant, as it should always be `reference_count == 1`, but having that not behind the pointer might make some things faster, I think?
    * A tricky thing here is that we wouldn't want to always have to check `first_index` when doing indexed operations, as that would make things slower, in the same way that a `VecDeque` is slower than a `Vec`. But there could just be two versions of each method, like `vec_get` and `deque_get` where the former just indexes directly into `data` while the latter makes use of `first_index` to act like a deque. At the VM level there could be `Value::VecList` and `Value::DequeList` that both internally use this same struct, but just call different versions of the indexed functions.
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
