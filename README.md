Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do

VM stuff:
* `Apply` instruction
  * takes 3 registers
    * target
    * function
    * list of arguments
  * functions will all start with n `Instruction::Argument` instructions, that provide symbol indexes to which the arguments passed to the function should will be bound. `Apply` will bind those arguments, then proceed through the rest of the instructions, until a `Return` instruction is reached, at which point it will copy the register referenced by the `Return` instruction into the `target` register.
  * oh, I guess we technically need lists before we can do this, to have something to pack the arguments into
    * can just make them naive CoW vectors for now
* lists (secretly vectors, but I'm gunna call them lists to make Lisp people mad >:D)
  * need to decide which persistent vector lib to use
    * maybe make my own?? The structure of RRB vectors feels overly restrictive, wanna try out some ideas of my own here
* strings
  * maybe implement as `Rc<&str>` rather than `String`?
* hashmaps
  * should test the DiffVec paradigm for hashmaps too

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
