#include<stdio.h>
typedef int* intptr;
int main()
{
  int x = 1;
  intptr y = &x;
  printf("%d\n",x);
  return 0;
}
