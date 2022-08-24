/* Secret File Download Client by Chuck Liang.
   For academic use only.
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
 
#define PORT 50021


int main(int argc, char **argv)
{
  int sockfd, ffd, result;
  FILE * sinstream, * soutstream;
  char buffer[1024];
  char buffer2[256];
  char c;
  struct sockaddr_in saddr;
  saddr.sin_family = AF_INET;
  saddr.sin_port = htons(PORT);
  if (argc > 1)
    saddr.sin_addr.s_addr = inet_addr(argv[1]);
  else 
    saddr.sin_addr.s_addr = inet_addr("147.4.150.248");

  sockfd = socket(AF_INET,SOCK_STREAM,0);
  result = connect(sockfd,(struct sockaddr *)&saddr,sizeof(saddr));    
  if (result == -1) { perror("failed to connect to server\n"); exit(1); }
  sinstream = fdopen(sockfd,"r");
  soutstream = fdopen(dup(sockfd),"w");
  
  /* wait for server synch byte */
  c = (char) 0;
  while (c != (char) 0x55) read(sockfd,&c,1);

  /* send ack to server */
  c = (char) 0xAA;
  write(sockfd,&c,1);

  /* determine file to download */
  printf("name of file to download (must start with 's'): ");
  scanf("%s",buffer2);
  fprintf(soutstream,"%s",buffer2);  fflush(soutstream);  
  
  /* read ack, 0 not ok, 1 ok)  */
  fscanf(sinstream,"%c",&c);
  if (c == (char) 1) 
    {
      ffd = creat(buffer2,0755);     /* open local file */
      result = 1;
      while (result>0)
	{ 
	  result = read(sockfd,&buffer,1024);
	  if (result>0) write(ffd,&buffer,result);
	} // while
      printf("file downloaded.\n");
    } // if file exists on server
  else // no file
    { perror("file open failed on server\n"); }
  close(ffd);
  close(sockfd);     
  exit(0);
}
