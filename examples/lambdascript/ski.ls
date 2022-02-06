define I = lambda x.x;
define K = lambda x.lambda y.x;
define S = lambda x.lambda y.lambda z.(x z (y z));
define True = K;
define False = (K I);
(K 1 2);
(K I 1 2);
((lambda x.lambda y.x) y 4);
((lambda x.lambda y.(x (lambda x.x))) y 4);
(S K I);
(S I I);
(S (K I) K);

