define I = lambda x.x;
define K = lambda x.lambda y.x;
define S = lambda x.lambda y.lambda z.(x z (y z));
define True = K;
define False = (K I); // applications require ()'s here
define lazy INFINITY = (lambda x.x x) (lambda x.x x);
// all definitions are reduced to "weak-head normal form" except define lazy

(lambda x.lambda y.x y) y 4;
(lambda x.lambda y.x (lambda x.x)) (z y) 4;
lambda x.x 1;
(lambda x.x) 1;
// in an applications, all lambdas require ()'s
(lambda x.lambda y.y x) 1 (lambda u.u);
(lambda x.y) INFINITY; // this uses CBN, so no infinite reduction

K 1 2;
K I 1 2;
S K I;
S I I;
S (K I) K;

define Ifelse = lambda c a b.(c a b);
Ifelse True 1 2;
Ifelse False 1 2;
define AND = lambda a b.(a b False);
define OR = lambda a b.(a True b);
AND False False;
AND False True;
AND True False;
AND True True;

I (I I);
weak (I (I I));

S I (K I);
weak (S I (K I));
define istrue = True;
