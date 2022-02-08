// Programming with pure lambda terms, with full beta-normalization via CBN.

// Baisc combinators:
define I = lambda x.x;
define K = lambda x.lambda y.x;
define S = lambda x.lambda y.lambda z.x z (y z);

// Booleans and if-else
define TRUE = lambda x.lambda y.x;     // K A B = A
define FALSE = lambda x.lambda y.y;  // (K I) A B = B
define IF = lambda b.lambda x.lambda y.b x y;
define NOT = lambda b.IF b FALSE TRUE;
define OR = lambda a.lambda b.IF a TRUE b;
define AND = lambda a.lambda b.IF a b FALSE;

// Arithmetic
define ZERO = lambda f.lambda x.x;
define ISZERO = lambda n.n (lambda z.FALSE) TRUE;
define ONE = lambda f.lambda x.f x;
define PLUS = lambda m.lambda n.lambda f.lambda x.m f (n f x);
define TIMES = lambda m.lambda n.lambda f.lambda x.m (n f) x;
define PRED = lambda n.lambda f.lambda x.n (lambda g.lambda h.h (g f)) (lambda u.x) (lambda u.u);
define SUBTRACT = lambda m.lambda n. n PRED m;
define LEQ = lambda m.lambda n. ISZERO (SUBTRACT m n);  // <=
define EQUALS = lambda m.lambda n.AND (LEQ m n) (LEQ n m);
LEQ ONE (PLUS ONE ONE);   //true
LEQ (PLUS ONE ONE) ONE;  // false
EQUALS ONE (TIMES ONE ONE); // true

// Data structure (pairs, linked-list = nested pairs)
define CONS = lambda a.lambda b.lambda c.IF c a b; //pair constructor
define CAR = lambda c.c TRUE;  // first element of cons pair
define CDR = lambda c.c FALSE; // second element of cons pair
define NIL = FALSE;   // represents empty list
define ISNIL = lambda p.p (lambda a.lambda b.lambda z.FALSE) TRUE;

// Recursion (loops) and divergence.
define lazy INFINITY = (lambda x.(x x)) (lambda x.(x x));
define lazy FIX = lambda m.(lambda x.m (x x)) (lambda y.m (y y));

// A sample linked list
define TWO = PLUS ONE ONE;
define THREE = PLUS TWO ONE;
define FIVE = PLUS TWO THREE;
define P = (CONS TWO (CONS THREE (CONS FIVE NIL)));

SUBTRACT FIVE THREE;

I 1;
K 1;     // lambda y.1
K 1 2;   // 1
K I;     // lambda x.lambda y.y
K I 2;   // lambda y.y
K I 2 3; // 3
S K I;   
S I I;

IF (AND (NOT FALSE) TRUE) 1 2;  // 1

// linked lists:
CAR (CDR (CDR P));  // 5  // in oop languages, M.cdr().cdr().car()
ISNIL NIL;
ISNIL P;

// recursive function to sum all numbers in list L:
define lazy SUM = FIX (lambda f.lambda L. IF (ISNIL L) ZERO (PLUS (CAR L) (f (CDR L))));

//SUM P;  // reduces to church numeral 10 (do in interactive mode)

TIMES (PLUS ONE ONE) ZERO; // 2*0=0, 0 in church numeral is lambda f.lambda x.x

let x = ONE in (PLUS x x); // 2 in church numeral
// the above is expanded by the parser to ((lambda x.PLUS x x) ONE)

// static scoping: 
let x = 1 in 
  (let f = lambda y.x in 
    (let x = 2 in (f 0)));  // 1, since lambda calc implies static scoping.
                            // if 2 was printed, then it's dynamically scoped
			    // and something went horribly wrong.
/* the above let-expression expands to:
(λx.(λf.(λx.f 0) 2) (λy.x)) 1
 =>  (λf.(λx.f 0) 2) (λy.1)
 =>  (λx.(λy.1) 0) 2
 =>  (λy.1) 0
 =>  1

lambda calculus version of C program:
int x = 1;
int f(int y) { return x; }
int main()  {
  int x = 2;
  return f(0);
} // main returns 1 under static scoping, 2 under dynamic scoping.

C and virtually all languages are statically scoped.
*/
