#include<stdio.h>
#include<stdlib.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <unistd.h>
#include <netdb.h>
#include <arpa/inet.h>

#define BUFSIZE 1024

#define SPORT 50021

// argv[1] is key file
int main(int argc, char* argv[])
{
  int infd1, infd2, outfd, i;   // file descriptors
  int cfd, sfd;  // socket file descriptors
  int n, r, j;
  char c;
  unsigned char buffer[BUFSIZE];
  unsigned char key[BUFSIZE];
  char sbuf[128];

  struct sockaddr_in saddr;
  struct sockaddr_in caddr;
  saddr.sin_family = AF_INET;
  saddr.sin_addr.s_addr = htonl(INADDR_ANY); // wildcard
  saddr.sin_port = htons(SPORT);

  if (argc != 2) { perror("wrong number of arguments\n"); exit(1); }
  infd2 = open(argv[1],O_RDONLY,0);
  r = read(infd2,key,BUFSIZE);
  if (r < BUFSIZE) exit(1);


  sfd = socket(AF_INET,SOCK_STREAM,0);  // tcp
  bind(sfd,(struct sockaddr*)&saddr,sizeof(saddr));
  listen(sfd,16);

  // main server loop
  while (1)
    {
      i = sizeof(caddr);
      cfd = accept(sfd,(struct sockaddr*)&caddr,&i);
      if (cfd < 0) exit(1);

      // communicating
      r = read(cfd,sbuf,127);
      sbuf[r] = 0;  // 0-terminates string.
      infd1 = open(argv[1],O_RDONLY,0);
      if (infd1 < 0) // file non-extent
	{
	  c = (char)0;
	  write(cfd,&c,sizeof(char));
	}
      else // serve file after sending c=1
	{
	  c = (char)1;
	  write(cfd,&c,sizeof(char));        
	  r = BUFSIZE;
          while (r==BUFSIZE)
	    {
	      r = read(infd1,buffer,BUFSIZE);
	      if (r>0) 
		{ // xor and send over socket
		  for(i=0;i<r;i++) 
		    buffer[i] = buffer[i] ^ key[i];
		  write(cfd,buffer,r);
		}
	    }
	}
      close(infd1);
      close(cfd);
    }
  exit(0);
}
