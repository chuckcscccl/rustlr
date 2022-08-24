#include<stdio.h>
#include<stdlib.h>
#include<string.h>

// OVOAP Version 7E4: Feistel Cipher variant: runs on 64-bit fixed blocks

#define ROUNDS 4
#define VALSIZE 4
#define KEYSIZE 124

// ith key definition: byte index to rotate to
unsigned char KEYI[ROUNDS];// starting byte index (wraping) of each KEYi (0-255)
unsigned char KEY[KEYSIZE];

void randomize(unsigned char buf[], int len, int max)
{
  for(int i=0;i<len;i++) buf[i] = rand()%max;
}

// round function: takes 32bit half xor with key at offset,
// which tells it to use the ith version of the key. boolean rev applies
// key offsets in reverse order (for decryption)
void F(int i, unsigned char val[VALSIZE], unsigned char outbuf[VALSIZE], int rev)
{
  // for the ith key: lookup starting bit index in KEYI, then align i with
  // KEY[i] and bitwise xor:
  int si = KEYI[i];  // starting byte index
  if (rev) si = KEYI[ROUNDS-1-i];
  for(int j=0;j<VALSIZE;j++)
    outbuf[j] = val[j] ^ KEY[(j+si)%KEYSIZE];
}//F(i)

// encrypt one block
void crypt1(const unsigned char LR[VALSIZE*2], unsigned char out[VALSIZE*2],int rev)
{
  unsigned char Li[VALSIZE];
  unsigned char Ri[VALSIZE];
  unsigned char Lnext[VALSIZE];
  unsigned char Rnext[VALSIZE];
  memcpy(Li,LR,VALSIZE);  memcpy(Ri,LR+VALSIZE,VALSIZE);
  for(int i=0;i<ROUNDS;i++)
    {
      memcpy(Lnext,Ri,VALSIZE); // Lnext = Ri
      F(i,Ri,Rnext,rev); // F(Ri,Keyi)  
      for (int k=0;k<VALSIZE;k++) Rnext[k] = Li[k] ^ Rnext[k]; // L^F(Ri,Keyi)
      memcpy(Li,Lnext,VALSIZE); memcpy(Ri,Rnext,VALSIZE);
    }
  memcpy(out,Rnext,VALSIZE);
  memcpy(out+VALSIZE,Lnext,VALSIZE);  // ciphertext is (Rnext,Lnext)
}//encrypt1

void crypt(const unsigned char M[], int len, unsigned char C[], int rev)
{
  if ((len % (VALSIZE*2)) !=0) {printf("bad length\n"); return; }
  for (int offset=0;offset<len;offset+=VALSIZE*2)
    {
      crypt1(M+offset,C+offset,rev);
    }
}

int main(int argc, char* argv[])
{
  int s = 7;
  if (argc>1) s = atoi(argv[1]);
  srand(s); // seed random number generator
  randomize(KEY,KEYSIZE,256);
  randomize(KEYI,ROUNDS,KEYSIZE);  // generates keys
  char *msg = "abcdefg"; "xyzzy20";
  unsigned char m[VALSIZE*2];
  unsigned char m2[VALSIZE*2];  
  memcpy(m,msg,strlen(msg)+1);
  unsigned char c[VALSIZE*2]; // ciphertext
  crypt1(m,c,0);
  crypt1(c,m2,1);
  for(int i=0;i<VALSIZE*2;i++) printf("%c ",(char)m[i]); printf("\n");        
  for(int i=0;i<VALSIZE*2;i++) printf("%d ",m[i]); printf("\n");
  for(int i=0;i<VALSIZE*2;i++) printf("%d ",c[i]); printf("\n");
  for(int i=0;i<VALSIZE*2;i++) printf("%d ",m2[i]); printf("\n");

  msg = "Sam secretly loves this class.";
  int len = strlen(msg)+1;
  if (len%(VALSIZE*2)!=0)
    len += VALSIZE*2 - len%(VALSIZE*2);
  unsigned char* MS = (unsigned char*)malloc(len);
  unsigned char* CS = (unsigned char*)malloc(len);
  unsigned char* MCS = (unsigned char*)malloc(len);    
  memcpy(MS,msg,strlen(msg)+1);
  crypt1(MS,CS,0);
  crypt1(CS,MCS,1);
  printf("Original message: %s\n",(char*)MS);
  
  return 0;
}//main
