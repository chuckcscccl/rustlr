## Lambdascript

**Lambdascript** executes beta-reduction steps on untyped lambda
terms.  It is not a high-performance implementation of lambda
calculus. Rather, the tool serves three primary purposes, all of which
are illustrational or educational in nature:

  1. It demonstrates the usage of the **[rustlr](https://docs.rs/rustlr/latest/rustlr/index.html)** *parser generator*.  The LALR(1) grammar, in rustlr format,
  is given [here](https://cs.hofstra.edu/~cscccl/rustlr_project/lambdascript/untyped.grammar).

  2. For introductory level students in a programming languages class, the
  tools shows every step of beta reduction, including alpha-conversions where
  necessary, in reducing a term to normal form.  It includes both full
  beta-normalization using the normal order (call-by-name) strategy as well 
  weak-head normalization using call-by-value.  Definitions can be made to
  define terms such as S, K, I.

  3. For more advanced students, the source code of the program demonstrates
  how lambda terms can be represented in abstract syntax and how
  reductions can be implemented.

### Usage
The program should be installed as an executable: **`cargo install lambdascript`**.  The program can read from a script or from stdin. expressions and defintions are separated by ;.  Here's an example of reading and evaluating from
stdin

```
<<< (lambda x.x (lambda y.x y)) y;
(λx.x (λy.x y)) y
 =>  y (λy1.y y1)
```

Given a file [simple.ls](https://cs.hofstra.edu/~cscccl/rustlr_project/lambdascript/simple.ls) with the following contents:
```
define I = lambda x.x;
define K = lambda x.lambda y.x;
define lazy INFINITY = (lambda x.x x) (lambda x.x x);

K I INFINITY x;
```
**`lambdascript simple.ls`** produces the following output:
```
K I INFINITY x
= (λxλy.x) I INFINITY x
 =>  (λy.I) INFINITY x
= (λyλx.x) INFINITY x
 =>  (λx.x) x
 =>  x
```
The reduction terminated because the default uses CBN evaluation.  If the
the last line of the file was replaced with `weak (K I INFINITY x)`, then
weak-head reduction using CBV will take place, 
resulting in an infinite loop.  There will likewise be an infinite loop if
`lazy` was missing from the definition of `INFINITY`.

As this tool is used actively in the classroom, each release will have
a **limited lifetime**: after a certain period it will cease to work until
a new version is released.