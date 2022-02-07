// Sample Solutions to Lambda Calculus Homework in Lambdascript.

// 2.
2;  // 2 is evaluated into 2, which is already a normal form
(lambda x.lambda y. (y x)) u (lambda x.x);


// 3. Basic combinators:
3;
define I = lambda x.x;
define K = lambda x.lambda y.x;
define S = lambda x.lambda y.lambda z.x z (y z);

K I I;
S I K;
S K (K I);
S (K I);

// note that the lambdascript program does call-by-name by default.
weak (S (K I));  // this does "weak head reduction using call-by-value"

// 4. Arithmetic
4;
define ZERO = K I;
define ONE = lambda f.lambda x.f x;
define PLUS = lambda m.lambda n.lambda f.lambda x.m f (n f x);
define TIMES = lambda m.lambda n.lambda f.lambda x.m (n f) x;
define TWO = PLUS ONE ONE;

PLUS TWO ONE;
TIMES ZERO TWO;

// 4b.
4 b;  // a space is needed otherwise there's parser error
define EXPT = lambda m.lambda n.n m;
define TREE = PLUS ONE TWO;

EXPT TREE TWO;


// 5. Booleans and if-else
5;
define TRUE = K;     // K A B = A
define FALSE = K I;  // (K I) A B = B
define IF = lambda b.lambda x.lambda y.b x y;
define NOT = lambda b.IF b FALSE TRUE;
define OR = lambda a.lambda b.IF a TRUE b;
define AND = lambda a.lambda b.IF a b FALSE;
OR;
OR FALSE FALSE;
OR FALSE TRUE;
OR TRUE FALSE;
OR TRUE TRUE;


6;
define NOT = lambda n.n FALSE TRUE;
NOT;
NOT FALSE;
NOT TRUE;
NOT (NOT TRUE);  // "no, no it can't be true!"  but it is.

7;
IF TRUE 1 2;
IF FALSE 1 2;


// Data structure (pairs, linked-list = nested pairs)
define PAIR = lambda a.lambda b.lambda c.IF c a b; // called "CONS"
define FST = lambda c.c TRUE;  // first element of cons pair, "CAR"
define SND = lambda c.c FALSE; // second element of cons pair, "CDR"
define NIL = FALSE;   // represents empty list
define ISNIL = lambda p.p (lambda a.lambda b.lambda z.FALSE) TRUE;

8;
define M = PAIR a (PAIR b c);
M;
FST M;
SND M;
SND (SND M);

9;
define ISZERO = lambda n.n (lambda z.FALSE) TRUE;
ISZERO;
ISZERO (TIMES ONE ZERO);
ISZERO (PLUS ONE ZERO);

10;
still working on it;
define lazy INFINITY = (lambda x.x x) (lambda x.x x);
// without "lazy" it will go into an infinite loop

//  weak (INFINITY);   // uncomment at your own risk...
