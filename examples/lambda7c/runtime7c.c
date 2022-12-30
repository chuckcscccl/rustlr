#include<stdio.h>
#include<stdlib.h>
#include<string.h>
#include<stdint.h>

typedef int32_t i32;

////////////////numhash data structure maps ints to ints
//////////////// use linear probing closed hashtable
#define TABLESIZE 65536
#define UMAX 0xffffffffffffffff
#define IMAX 0x7fffffff
struct numhash {
  uint64_t key[TABLESIZE];
  int val[TABLESIZE];
  int keylocs[TABLESIZE]; // locations of keys, maybe inactive
  int keys;               // number of keys
}; // numhash

static struct numhash MemMap; // global memory map

void reference_map_init() {
  memset(MemMap.key,-1,TABLESIZE*8); // set all keys to UMAX
}
//int hash(uint64_t k) { return k % TABLESIZE; }
// int rehash(int h) { return (h+1)%TABLESIZE; }
int findhashslot(uint64_t k) {
  int hash = k%TABLESIZE; // original hash
  int i = 0;
  while (MemMap.key[hash]!=k && MemMap.key[hash]!=UMAX && i<TABLESIZE) {
    hash = (hash+1)%TABLESIZE; //rehash
    i++;
  }
  if (i==TABLESIZE) {
    printf("RUNTIME ERROR: Reference Map Full (65536)\n");
    exit(3);
  }
  return hash;
}
void hash_set(uint64_t k, int v) {
  int hash = findhashslot(k);
  if (MemMap.key[hash]==UMAX) MemMap.keylocs[MemMap.keys++]=hash;
  MemMap.key[hash]= k;  MemMap.val[hash] = v;
}
int hash_get(uint64_t k) {
  int hash = findhashslot(k);
  if (MemMap.key[hash]!=k) return IMAX;
  else return MemMap.val[hash];
}
int hash_remove(uint64_t k) {
  int hash = findhashslot(k);
  if (MemMap.key[hash]!=k) return 0;
  MemMap.val[hash] = IMAX;
  return 1;
}
/////////////// hash table

///// use as a reference counter

int reference_register(void* address) { //
  int hash = findhashslot((uint64_t)address);
  if (MemMap.key[hash]==UMAX) {
     MemMap.keylocs[MemMap.keys++]=hash;
     MemMap.key[hash] = (uint64_t)address;  MemMap.val[hash] = 0;
     return 0; // new reference count
  }
  //MemMap.val[hash]+=1;
  //return MemMap.val[hash];
  return 0x80000000;
}//reference_register

int reference_inc(void* address) { //
  if (address==0) return 0;
  int hash = findhashslot((uint64_t)address);
  if (MemMap.key[hash]==UMAX) {
     MemMap.keylocs[MemMap.keys++]=hash;
     MemMap.key[hash] = (uint64_t)address;  MemMap.val[hash] = 1;
     return 1; // new reference count
  }
  MemMap.val[hash]+=1;
  return MemMap.val[hash];
}//hash_inc

int reference_dec(void* address) {  // prepare to free when counter is zero
  if (address==0) return 0;  
  int hash = findhashslot((uint64_t)address);
  if (MemMap.key[hash]==UMAX) return -1; // nothing happened.
  int v = MemMap.val[hash];
  if (v<1) return v; // do nothing
  MemMap.val[hash] = v-1;
  if (0 == v-1) {    free(address);    }   // !! free called here !!
  return v-1;  // new reference count
}//reference_dec


// *** this doesn't work if a closure contains pointers to other closures.
// solution: compiler must generate a specific *destructor* for every struct
// that it creates for each closure.  The destructor must weaken (decrement)
// every pointer inside the struct before the struct is freed.

// so the current solution is strictly second, not higher ordered.

// need:

int decrement_rc(void *addr, void (*destructor)(void*) ) {
  if (addr==0) return 0;  
  int hash = findhashslot((uint64_t)addr);
  if (MemMap.key[hash]==UMAX) return -1; // nothing happened.
  int v = MemMap.val[hash];
  if (v<1) return v; // do nothing
  MemMap.val[hash] = v-1;  // decrement reference counter for this addr
  if (v==1) { destructor(addr); free(addr); } // note indirect recursion
  // this will do a tree traversal, very expensive in worst case.
  return v-1;
}//decrement_rc
// instead of reference_dec, must call decrement_rc with destructor, which
// must be written in LLVM at compile time.



void* rcmalloc(int64_t n) {
  if (n<1) return 0; // null;
  void* addr = malloc(n);
  reference_inc(addr);
  return addr;
}//rcmalloc


/// to prevent cycles, programmers must be able to call weaken on an address
////  weaken is called when?
/*
    define a = lambda ...
    define b = lambda ...
    // will a have b inside it's closure?  order doesn't matter?
    // this can't happen: b doesn't exist when the runtime close of a is
    // formed.
    intrinsic function weaken(pointer) will just decrease ref count?
*/



i32 lambda7c_cin()
{
  i32 x;
  printf(">> ");
  scanf/*_s*/("%d",&x);
  //scanf_s("%d",&x);  
  return x;
}//lambda7c_cin()

i32 lambda7c_expt(i32 x, i32 n)  // x**n
{
  i32 ax = 1;
  i32 fct = x;
  while (n>0)
    {
      if (n%2==1) ax*=fct;
      fct *=fct;
      n/=2;
    }
  return ax;          
}


i32 lambda7c_not(i32 x)
{
  if (x==0) return 1; else return 0;
}
i32 lambda7c_neg(i32 x)
{
  return -1*x;
}

void lambda7c_printint(i32 x)
{
  printf("%d",x);
}
void lambda7c_printfloat(double x)
{
  printf("%f",x);
}
void lambda7c_printstr(char* x)
{
  printf("%s",x);
}
void lambda7c_newline()
{
  printf("\n");
}

void exit_error(int errcode, int line) //negative line if unavailable
{
  switch (errcode) {
  case 3: printf("Array index out of bounds"); break;
  default: printf("Unspecified runtime error");
  }//switch
  printf(", program aborted.\n");
  if (line>=0) printf("This error likely originated from line %d of the source code",line);
  exit(errcode);
}

void check_index(int index, int size, int line) {
  if (index<0 || index>=size) exit_error(3,line);
}


// fill array with initial value
void fillarray_int(int* A, int init, int size)
{
  for (int i=0;i<size;i++) A[i]=init;
}
void fillarray_double(double* A, double init, int size)
{
  for (int i=0;i<size;i++) A[i]=init;  
}
void fillarray_ptr(void** A, void* init, int size)
{
  for (int i=0;i<size;i++) A[i]=init;    
}
