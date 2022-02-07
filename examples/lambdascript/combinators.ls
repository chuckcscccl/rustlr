// Programming with pure lambda terms, with full beta and weak head reduction
// default define means reduce defined term to weak-head normal form.

define I = lambda x.x;
define K = lambda x.lambda y.x;
define S = lambda x.lambda y.lambda z.(x z (y z));
define True = K;
define False = (K I);
define IF = lambda b.lambda x.lambda y.(b x y);
define NOT = lambda b.(IF b False True);
define OR = lambda a.lambda b.(IF a True b);
define AND = lambda a.lambda b.(IF a b False);
define CONS = lambda a.lambda b.lambda c.(IF c a b);
define CAR = lambda c.(c True);
define CDR = lambda c.(c False);
define lazy FIX = lambda M.((lambda x.(M (x x))) (lambda y.(M (y y))));
define Zero = False;
define One = lambda f.lambda x.(f x);
define PLUS = lambda m.lambda n.lambda f.lambda x.(m f (n f x));
define TIMES = lambda m.lambda n.lambda f.lambda x.(m (n f) x);
//define lazy INFINITY = (lambda x.(x x)) (lambda x.(x x));

define NULL = False;
define M = (CONS 2 (CONS 3 (CONS 5 (CONS 7 NULL))));

(K 1);     // lambda y.1
(K 1 2);   // 1
(K I);     // lambda x.lambda y.y
(K I 2);   // lambda y.y
(K I 2 3); // 3
//weak (S K I); // weak head normal form
(S K I);   // full normal form
(S K I I); // lambda x.x
(S K I 0); // 0

// booleans and if-else
(IF (AND (NOT False) True) 1 2);  // 1

// linked lists:
(CAR (CDR (CDR M)));  // 5

// arithmetic
(PLUS One One);  // 2 in church numeral

(TIMES (PLUS One One) Zero 1 0); // 0: 2*0=0!

// static scoping: 
let x = 1 in 
  (let f = lambda y.x in 
    (let x = 2 in (f 0)));  // 1, since lambda calc implies static scoping.
                            // if 2 was printed, then it's dynamically scoped
			    // and something went horribly wrong.
