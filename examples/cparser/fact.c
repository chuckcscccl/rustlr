#include<stdio.h>

// three versions of n!

int fact1(int n) 
  { if (n<2) return 1; else return n*fact1(n-1); }

// Scheme version of fact1:
// (define (fact1 n) (if (< n 2) 1 (* n (fact1 (- n 1)))))


int fact2(int n)
  { int ax = 1; 
    while (n>1)
     { ax = ax * n;
       n = n-1;
     }
     return ax;
  }        


int fact3(int n, int ax)
  { if (n<2) return ax; else return fact3(n-1,ax*n); }


// Scheme version of fact3:
// (define (fact3 n ax) (if (< n 2) ax (fact3 (- n 1) (* ax n))))


// bubblesort just for kicks
void bubble(int* A, int len)
{
  int i, k;
  for(i=0;i<len-1;i++)
    for(k=0;k<len-1-i;k++)
      {
        if (A[k]>A[k+1]) {
          A[k] += A[k+1];
          A[k+1] = A[k] - A[k+1];
          A[k] -= A[k+1];
        }
      }
}//bubble


int main()
{
  printf("the factorial of 5 is %d\n", fact1(5));
  printf("with tail recursion it's also %d\n", fact3(5,1));
  printf("without recursion it's still %d\n", fact2(5));
}
