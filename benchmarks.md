simple decrement from 10,000,000 to 0 test:
  * Pidgin vm
    * code:
      ```rust
      use block_macros::block;
      use pidgin::{Block, EvaluationState, GenericInstruction::*, Value};
      let time = std::time::Instant::now();
      let program = block![
        Const(0, 100000000),
        Const(
          1,
          Value::composite_fn(
            1,
            block![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)]
          )
        ),
        Call(0, 1, 1),
        StealArgument(0),
      ];
      let mut state = EvaluationState::new(program);
      state.evaluate().unwrap();
      println!("{}", time.elapsed().as_secs_f64())
      ```
    * runtime: 1.796 seconds (with release build, debug build is much slower)
  * babashka
    * code:
      ```clj
      (time (loop [x 100000000] (if (pos? x) (recur (dec x)) x)))
      ```
    * runtime: 4.528 seconds
  * clojure (JVM)
    * code:
      ```clj
      (time (loop [x 100000000] (if (pos? x) (recur (dec x)) x)))
      ```
    * runtime:
      * 0.034 seconds
  * chez scheme
    * code:
      ```scheme
      (define (f x)
        (if (positive? x)
          (f (- x 1))
          x))
      (time (f 100000000))
      ```
    * runtime: 0.226 seconds
  * javascript
    * code:
      ```javascript
      let start = Date.now();
      let x = (() => {
        let x = 100000000;
        while (x > 0) {
          x--;
        }
        return x;
      })();
      console.log(x, Date.now() - start);
      ```
    * runtime: 0.069 seconds