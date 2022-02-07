define I = lambda x.x;
define K = lambda x.lambda y.x;
define lazy INFINITY = (lambda x.x x) (lambda x.x x);

K I INFINITY x;
