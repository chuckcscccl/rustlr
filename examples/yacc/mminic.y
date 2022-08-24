/***********************************/
/*                                 */
/* COMPILATEUR DE MINI C POUR MIPS */
/*                                 */
/*    Jacques Farré - ESSI         */
/*                                 */
/*         février 2004            */
/*                                 */
/***********************************/

/****************************************************
  C'est une grammaire qui ressemble à celle de C, 
  mais ce n'est pas une vraie grammaire de C
****************************************************/

%{

#include <stdio.h>
#include <iostream>
using namespace std;

#include "arbre.h"

/* définis dans Lex */
extern int yylex (void);
extern FILE* yyin;

static GenreVariable genre_courant;
static const TypeSimple* type_base;

static ListeDeTypes* parametres_courants;

%}

%union {
  int              	ve;
  const char*           st;
  NoeudInstr*	        in;
  NoeudExpr*	        ex;
  NoeudDesig*     	ds;
  ListeArguments*	ar;
  ListeDeVariables*	va;
  ListeInstructions*	le;
  struct {
    Type*         typ;
    const char*   var;
    Fonction*     fct;
    ListeDeTypes* par;
  } de;
  struct {
    const Type* typ;
    Fonction*   fct;
  } di;
  struct { // pour sauvegarder les variables globales
    const TypeSimple* typ;
    Fonction*         fct;
    ListeDeTypes*     par;
    GenreVariable     gen;
  } fo;
}

%type<di> declaration_initiale declarateur
%type<de> pdecl tdecl sdecl
%type<ve> dim
%type<fo> ident_fonc 
%type<st> ident_var
%type<le> liste_instr corps
%type<in> instruction instruction_simple si tantque repeter retour bloc
%type<ex> condition expression facteur primaire
%type<ar> arguments
%type<va> variables
%type<ds> designation denotation

%token GOTO IF ELSE WHILE DO BREAK RETURN INT VOID PRINT READ

%token<ve> CONST
%token<st> IDENT STRING

%right '='
%left OU
%left ET
%nonassoc EQU NEQ GEQ LEQ '<' '>'
%left SLL SLR
%left '+' '-'
%left '*' '/' '%'
%right '!'
%right PREFIXE
%left '['

%%
progr
  : declarations
  ;

declarations
  : declaration
  | declarations declaration
  ;

declaration
  : declarations_simples
  | definition_fonction
  ;

definition_fonction
  : declaration_fonction '{' corps '}'	
  ;

declaration_fonction
  : declaration_initiale		
  ;

declarations_simples
  : declaration_initiale suite_decl ';'	
  ;

suite_decl
  : /* vide */				
  | ',' declarateur suite_decl		
  ;

declaration_initiale
  : type_initial declarateur		
  ;

type_initial
  : INT					
  | VOID				
  ;

declarateur 
  : pdecl			
  ;

pdecl
  : '*' pdecl				
  | tdecl				
  ;

tdecl
  : sdecl				
  | tdecl dim				
  ;

dim
  : '[' ']'				
  | '[' CONST ']'			
  ;

sdecl
  : /* vide */ 				
  | ident_var			        
  | ident_fonc '(' parametres ')' 
  | '(' pdecl ')'			
  ;

ident_var
  : IDENT			        
  ;

ident_fonc
  : IDENT			
  ;

parametres 
  : /* vide */
  | liste_parametres
  ;

liste_parametres
  : declaration_parametre			
  | liste_parametres ',' declaration_parametre	
  ;

declaration_parametre
  : declaration_initiale	
  ;

corps
  : decl_locales liste_instr		
  | liste_instr				
  | decl_locales			
  ;

decl_locales
  : declarations_simples
  | decl_locales declarations_simples
  ;

liste_instr
  : instruction				
  | liste_instr instruction		
  ;

instruction 
  : bloc				
  | si					
  | tantque				
  | repeter ';'				
  | instruction_simple ';'	        
  | expression ';'			
  | error ';'		                
  | ';'       	                	
  ;

instruction_simple
  : retour				
  | BREAK 			
  | PRINT '(' arguments ')' 		
  | READ '(' variables ')'		
  ;

si
  : IF condition instruction		
  | IF condition instruction ELSE instruction 	
  ;	

tantque
  : while condition instruction		
  ;

while
  : WHILE 				
  ;

repeter
  : do bloc WHILE condition 	 	
  ;

do
  : DO	 				
  ; 

retour
  : RETURN 				
  | RETURN expression 			
  ;

bloc
  : '{' liste_instr '}'       		
  | '{' '}' 				
  ;

condition
  : '(' expression ')'			
  ;

expression
  : designation '=' expression  	
  | expression OU expression		
  | expression ET expression		
  | expression EQU expression		
  | expression NEQ expression		
  | expression '<' expression		
  | expression LEQ expression		
  | expression '>' expression		
  | expression GEQ expression		
  | expression SLL expression		
  | expression SLR expression		
  | expression '+' expression		
  | expression '-' expression  		
  | expression '*' expression		
  | expression '/' expression		
  | expression '%' expression		
  | facteur				
  ;

facteur
  : CONST				
  | '-' CONST        
  | STRING                              
  | '&' designation  
  | '!' facteur				
  | '-' primaire     
  | primaire				
  ;

primaire
  : designation				
  | IDENT '(' ')'			
  | IDENT '(' arguments ')'		
  | '(' expression ')'			
  ;

designation
  : denotation				
  | '*' primaire 
  ;

denotation
  : IDENT				
  | primaire '[' expression ']'		
  ;

arguments
  : expression				
  | arguments ',' expression		
  ;
  
variables
  : designation				
  | variables ',' designation		
  ;
  
%%

//------------------------

void yyerror (char* msg) { 
  const char* symb = yytext [0] == '\n' ? "fin de ligne"
                   : yytext [0] == '\0' ? "fin de fichier"
	           : yytext;
  Erreur (msg, yylineno, symb);
}

//------------------------

int yyparse ();

bool analyser_source (const char* p) {
  int l = strlen (p);
  char* f = new  char [l + 3];
  strcpy (f, p); strcpy (f + l, ".c");
  yyin = fopen (f, "r");

  if (yyin == 0) {
    cerr << "impossible d'ouvrir " << f << '\n';
    return false;
  }

  fichier = f;
  erreurs = false;
  genre_courant = vglobale;
  yyparse ();
  return !erreurs;
}
