Early WIP programming language. Intended to be a Clojure-like Lisp with a more powerful metaprogramming system, compiling to a register-based VM. Intended to be radically extensible, easily integrable with the Rust ecosystem, and ideal for creating DSLs (both DSLs hosted on it's own runtime, and DSLs that compile to entirely separate languages/runtimes). Following up on [Quoot](https://github.com/Ella-Hoeppner/Quoot).

# to-do

VM stuff:
* lists (secretly vectors, but I'm gunna call them lists to make Lisp people mad >:D)
  * need to decide which persistent vector lib to use
    * maybe make my own?? The structure of RRB vectors feels overly restrictive, wanna try out some ideas of my own here
* strings
  * maybe implement as `Rc<&str>` rather than `String`?
* functions
  * I guess these can just be a vector of instructions?
    * but I guess in that case they'll need to decide which registers to use on the fly, that couldn't be fixed by the compiler...
      * I guess each function could operate on virtual registers numbered 0..n, and the compiler could just assign them to real registers as needed
        * I suppose the compiler could always keep track of the max used register, and assign the virtual registers 0..n to real registers (m+1)..(m+n+1) where n is the max used register
          * tail recursion seems easy enough, but I guess with mutually recursive functions this approach would eat up a lot of registers...
            * I guess maybe just provide some stuff to make trampolining easily? Would be nice if there were a way to do this automatically without having to bother the user...
        * Or it could assign them 1-by-1 to the smallest unused registers. Seems like that would involve more computation tho, and might be worse for cache locality (??)

Language stuff:
* Finish GSE (in its repo)
* Specify a default parser
* Start on a compiler
  * start with just arithmetic stuff
  * lists/vectors
    * can just start with the basic operations for now
      * push, pop, count, concat
  * functions
    * not entirely sure how stack tracing works in register-based VMs
      * is it even strictly necessary? Definitely want to be able to do it for debugging purposes, but it feels like if the compiler keeps good track of the active registers then it feels like you could just do a function application without needing to do the equivalent of pushing a stack frame
  * let bindings
    * should this just be a macro that expands to a function call?? this is how CL does afaik. I feel uncertain about whether this makes sense from a performance perspective, not sure tho. Maybe CL can get away with it because it's compiler is very advanced??