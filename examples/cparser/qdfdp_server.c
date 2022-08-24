/* Secret File Download Protocol Server by Chuck Liang
   For academic use only.

   Compile on sun with 
   gcc -O2 -lnsl -lsocket -lpthread sfdp_server.c -o sfdp_server
*/

#include<stdio.h>
#include<stdlib.h>
#include<sys/types.h>
#include<sys/socket.h>
#include<netinet/in.h>
#include<arpa/inet.h>
#include<unistd.h>
#include<netdb.h>
#include<fcntl.h>
#include<pthread.h>
#include <sys/time.h>

#define PORT 50021

void* sessionThread(void * pcfd);
int readable_timeo(int fd, int sec);  // from unp book

int main(int argc, char *argv[])
{ pthread_t sthread;   // session thread

  int i, stop, result1, result2;
  struct sockaddr_in saddr;  // server and client 
  struct sockaddr_in caddr;
  int ssockfd, csockfd;  // file descriptors for sockets
  int serverlen, clientlen;

  printf("s2: Secret File Download Server Started\n");

  ssockfd = socket(AF_INET, SOCK_STREAM,0);
  // assume success  - worry later

  saddr.sin_family = AF_INET;
  saddr.sin_addr.s_addr = htonl(INADDR_ANY);
  saddr.sin_port = htons(PORT);
  serverlen = sizeof(saddr);

  bind(ssockfd,(struct sockaddr *)&saddr, serverlen); // bind port
  listen(ssockfd,32); // 32 clients max - set TCP to passive state
  clientlen = sizeof(caddr);  // must do - inout parameter.
  
  for(;;)
  {
  // wait for connection, create session thread
  csockfd = accept(ssockfd,(struct sockaddr *)&caddr,&clientlen);  
  if (csockfd != -1)
    {  printf("s2: connection from %s\n",inet_ntoa(caddr.sin_addr));
       pthread_create(&sthread,NULL,sessionThread,&csockfd);
    }
  } // for

  close(ssockfd);
  close(csockfd);
  exit(0);
} // main


void* sessionThread(void * pcfd)
{ 
  int i, stop, result1, result2;
  int csockfd, ffd;  // file descriptors for socket and file.
  char buffer[1024];
  char buffer2[256];
  char c;
  csockfd = *((int *)pcfd); 

  /* get requested name of file */
  result1 = 0;
  result2 = readable_timeo(csockfd,30);  // set 30 second timeout
  if (result2>0) result1 = read(csockfd,&buffer2,256);
 if (result1 > 0) 
 { // got file name
  buffer2[result1] = (char) 0;
printf("got file name %s\n",buffer2);
  if (buffer2[result1] != (char) 0) buffer2[result1+1] = (char) 0;
  if (*buffer2 != 'q') ffd = -1;   /* only files starting with q are served */
   else  ffd = open(buffer2,O_RDONLY,0);  /* open local file */
  if (ffd == -1) 
     {  c = (char) 0;  // indicate non-existent file
        write(csockfd,&c,1);
     }  // ffd==-1
  else  // transfer file
     {
        c = (char) 1;
        write(csockfd,&c,1);  // confirm file exists
	result1 = 1;
	while (result1>0)
         {  result1 = read(ffd,&buffer,1024);
	    if (result1>0) write(csockfd,&buffer,result1);
	 }  // while 
     }  //ffd!=-1
 } // got file name
  close(ffd);
  close(csockfd);
} // sessionthread

// return -1 on error, 0 on timeout
int readable_timeo(int fd, int sec)  // from unp book
{
  fd_set rset;
  struct timeval tv;
  FD_ZERO(&rset);
  FD_SET(fd,&rset);
  tv.tv_sec = sec;
  tv.tv_usec = 0;
  return (select(fd+1,&rset,NULL,NULL,&tv));
} // readable_timeo
