/* How to use tail-recursion instead of loops.  In mathematics, all
computer programs are recursive functions (totally recursive for the
ones that terminate).  For- and while loops are equivalent to
simplified forms of recursion.  A function is tail-recursive if
nothing else is done after any recursive call (but be sure to read
expressions inside out: n*f(n-1) does not call f last, it multiplies last)

A smart compiler and even some interpreters can also recognize these
cases of recursion and compile them to efficiently excuting code where
recursive calls do not push new frames onto the runtime call stack.
The gcc and g++ compilers both do this at level 2 or above
optimization (use gcc with -O2).  The scheme interpreter certainly do
this, as well as Elm.  Be aware however, that Java, Python, C# and many
other languages do NOT optimize tail recursion.  One reason for this
is that it becomes harder to implement it along with dynamic dispatch,
where the version of the function being called is not determined at
compile time.

Even though you probably won't use scheme in real world programming,
the functional style of programming is becoming more important as it's
naturally stateless.  There will be situations where recursion is more
natural to use than loops, and so it is important to understand the
difference between good and bad ways to use recursion.  Tail recursive
programs are always equivalent to loops.

This program in C, with scheme/elm code in comments, shows you how to
convert a program that uses while loops into a tail-recursive program.

The basic idea is the following:

*** For every variable that might be assigned to a new value in a
while loop, add a new argument (parameter) to the function.  The
initial values of the function are the values of these arguments when
the function is initially called.  The function's body
should first test if boolean condition of the while loop is still true
before proceeding: if it's false it should execute the code that comes
after the loop. If the boolean is true it should execute the
body of the while loop and end with a recursive call to itself, with
new values for the parameters that changed inside the loop.
***
*/

#include<stdio.h>

// first example: function to calculate n!:
int fact1(int n)
{
  int ax = 1; // accumulator
  while (n>1)
    {
      ax = ax * n;
      n -= 1;
    }
  return ax;
}
// can be rewritten tail-recursively as
int fact(int n, int ax)
{
  if (n>1) 
    {
      return fact(n-1, ax*n);
    }
  else return ax;
}
// The function must be called with initial value 1 for ax.  C compilers
// will generate nearly identical code for the two functions.
// In this example, the entire body of the while loop is replaced by a
// tail recursive call.  The result of the call is immediately returned by
// the function, so it is tail recursive.  // In scheme, we write this as
// (define (fact n ax) (if (> n 1) (fact (- n 1) (* ax n)) ax))
// and in Elm:
// fact n ax = if n>1 then (fact (n-1) (ax*n)) else ax


// Function to calculate nth fibonacci number:

int fibw(int n)
{
  int a =1, b = 1;
  while (n > 2)
    {
      b = a+b;
      a = b-a;  // a will have b's original value
      n--;
    }
  return b;
}

// This can be rewritten as the following tail recursive function.  Since
// we want to encapsulate the intial values that should be passed into this
// function, we will define it as a function inside another function.

int fib(int n)
{
  int tfib(int n, int a, int b)  // inner function of fib
  {
    if (n<=2) return b;
    else return tfib(n-1, b, a+b);
  }
  
  return tfib(n,1,1);  // body of outer "public" function
}//fib
/* in Scheme: 
(define (fib n)
   (define (tfib n a b)
     (if (<= n 1) b (tfib (- n 1) b (+ a b))))
   (tfib n 1 1) ; body of outer fib function
)

in Elm: 
fib m =
  let fib2 n a b = if n<2 then b else fib2 (n-1) b (a+b)
  in
  fib2 m 0 1
*/

// Function to calcuate the greatest common divisor of two integers:

int gcd1(int a, int b)
{
  while (a!=0)
    {
      int tmp = a;
      a = b % a;
      b = tmp;  // b becomes original value of a
    }
  return b;
}
// tail recursively:

int gcd(int a, int b)
{ if (a==0) return b; else return gcd(b%a,a); }
// (define (gcd a b) (if (= a 0) b (gcd (remainder b a) a)))  ;scheme
// gcd a b = if a==0 then b else gcd (modBy a b) a  --elm

// Note that in this function, the only use of the tmp variable was to
// help preserve the original value of a before it was changed, because
// C does not allow simulataneous assignments (a,b=...).  But with the
// tail recursive call, that's in fact what we're doing.  Thus in the
// recursive version tmp is not needed.

// Calcuate m**n in log(n) steps by binary factorization of n:
// m**13 = m * m**4 * m**8.

int expt1(int m, int n)
{
  int factor = m;   // **binary factor of m
  int ax = 1;       // accumulator, default m**0
  while (n>0)
    {
      if (n%2==1) 
	{
	  ax = ax*factor;
	}
      factor = factor*factor; // m, m**2, m**4, m**8, etc..
      n = n/2;
    }
  return ax;
}

// tail recursively: here it looks a bit more different.  The while
// loop always changes the factor and n variables, but may not change
// ax depending on the condition.  Thus you should have an if statement
// that makes one of two recursive calls, both calls must be tail calls.
int expt(int m, int n)
{
  int iter(int n, int ax, int factor)
  {
    if (n<1) return ax;
    else if (n%2==1) return iter(n/2, ax*factor,factor*factor);
    else return iter(n/2,ax,factor*factor);
  }
  return iter(n,1,m);
}
/* in scheme:
(define (expt m n)
  (define (iter n ax fct)
     (cond ((< n 1)  ax)
           ((= 1 (remainder n 2) (iter (quotient n 2) (* ax fct) (* fct fct))))
	   (#t (iterm (quotient n 2) ax (* fct fct)))))
  (iter n 1 m))

I used a cond instead of if-else in the scheme version.  Also note that
in both the C and Scheme programs, the inner tail-recursive function does
not require m as a parameter, because it does not change.

Another way to write the inner iter is:
(define (iter n ax fct)
   (let ((newax (if (= 1 (remainder n 2)) (* ax fct) ax)))
      (if (< n 1) ax (iter (quotient n 2) newax (* fct fct)))))

I used a new variable for the new value of ax, which allowed me
to make a single recursive call.

In Elm:
expt m n = 
  let iter nn ax fct = let newax=(if 1==(modBy 2 nn) then (fct*ax) else ax) 
                       in
                       if nn<1 then ax else iter (nn//2) newax (fct*fct)
  in
  iter n 1 m

Syntactically, Elm will not allow you to use 'n' again as the inner
function's variable, which is rather annoying.  Elm also imports some
python-ish elements like the '//' for integer quotient and requires
proper identation and alignment of code.
*/

int main()
{
  printf("%d %d %d %d\n", fact(5,1), fib(10), gcd(8,12), expt(2,13));
  return 0;
}
