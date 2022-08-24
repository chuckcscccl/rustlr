/* sml.y
 * Grammar from Appendix B in _The_definition_of_standard_ML_,
 * Milner et al., 1997, ISBN 0-262-63181-4.
 */

%token WILDCARD  "..."
%token MATCH     "=>"
%token APPL      "->"

%token ABSTYPE   "abstype"
%token AND       "and"
%token ANDALSO   "andalso"
%token AS        "as"
%token CASE      "case"
%token DATATYPE  "datatype"
%token DO        "do"
%token ELSE      "else"
%token END       "end"
%token EXCEPTION "exception"
%token FN        "fn"
%token FUN       "fun"
%token HANDLE    "handle"
%token IF        "if"
%token IN        "in"
%token INFIX     "infix"
%token INFIXR    "infixr"
%token LET       "let"
%token LOCAL     "local"
%token NONFIX    "nonfix"
%token OF        "of"
%token OP        "op"
%token OPEN      "open"
%token ORELSE    "orelse"
%token RAISE     "raise"
%token REC       "rec"
%token THEN      "then"
%token TYPE      "type"
%token VAL       "val"
%token WITH      "with"
%token WITHTYPE  "withtype"
%token WHILE     "while"

%token VID TYVAR TYCON LAB STRID SCON DIGIT

%start dec

%%

/* Expressions and matches */
atexp:
    SCON
  | op VID
  | '{' exprow '}'
  | '{' '}'
  | '#' LAB
  | '(' ')'
  | '(' expcn2 ')'
  | '[' expcn ']'
  | '[' ']'
  | '(' expsn2 ')'
  | "let" dec "in" expsn "end"
  | '(' exp ')'
  ;
expcn:
    exp
  | expcn ',' exp
  ;
expcn2:
    exp ',' exp
  | expcn2 ',' exp
  ;
expsn:
    exp
  | expsn ';' exp
  ;
expsn2:
    exp ';' exp
  | expsn2 ';' exp
  ;

exprow:
    LAB '=' exp
  | exprow ',' LAB '=' exp
  ;

appexp:
    atexp
  | appexp atexp
  ;

infexp:
    appexp
  | infexp VID infexp
  ;

exp:
    infexp
  | exp ':' ty
  | exp "andalso" exp
  | exp "orelse" exp
  | exp "handle" match
  | "raise" exp
  | "if" exp "then" exp "else" exp
  | "while" exp "do" exp
  | "case" exp "of" match
  | "fn" match
  ;

match:
    mrule
  | match '|' mrule
  ;

mrule:
    pat "=>" exp
  ;

/* Declarations and bindings */
dec:
    "val" tyvarseq valbind
  | "fun" fvalbind
  | "type" typbind
  | "datatype" datbind
  | "datatype" datbind "withtype" typbind
  | "datatype" TYCON '=' "datatype" TYCON
  | "abstype" datbind "with" dec "end"
  | "abstype" datbind "withtype" typbind "with" dec "end"
  | "exception" exbind
  | "local" dec "in" dec "end"
  | "open" stridn
  | /* empty */
  | dec dec
  | dec ';' dec
  | "infix" vidn
  | "infix" DIGIT vidn
  | "infixr" vidn
  | "infixr" DIGIT vidn
  | "nonfix" vidn
  ;
stridn:
    STRID
  | stridn STRID
  ;
vidn:
    VID
  | vidn VID
  ;

valbind:
    pat '=' exp
  | valbind "and" pat '=' exp
  | "rec" valbind
  ;

fvalbind:
    mfvalbind
  | fvalbind "and" mfvalbind
  ;
mfvalbind:
    sfvalbind
  | mfvalbind '|' sfvalbind
  ;
sfvalbind:
    op VID atpatn '=' tyop exp
  ;
op:
    /* empty */
  | "op"
  ;
tyop:
    /* empty */
  | ':' ty
  ;
atpatn:
    atpat
  | atpatn atpat
  ;

typbind:
    tyvarseq TYCON '=' ty
  | typbind "and" tyvarseq TYCON '=' ty
  ;
tyvarseq:
    TYVAR
  | '(' tyvarcn ')'
  | '(' ')'
  | /* empty */
  ;
tyvarcn:
    TYVAR
  | tyvarcn ',' TYVAR
  ;

datbind:
    tyvarseq TYCON '=' conbind
  | datbind "and" tyvarseq TYCON '=' conbind
  ;

conbind:
    sconbind
  | conbind '|' sconbind
  ;
sconbind:
    op VID 
  | op VID "of" ty
  ;

exbind:
    sexbind
  | exbind "and" sexbind
  ;
sexbind:
    op VID
  | op VID "of" ty
  | op VID '=' op VID
  ;

/* Patterns */
atpat:
    '_'
  | SCON
  | op VID
  | '{' patrow '}'
  | '{' '}'
  | '(' ')'
  | '(' patcn2 ')'
  | '[' ']'
  | '[' patcn ']'
  | '(' pat ')'
  ;
patcn:
    pat
  | patcn ',' pat
  ;
patcn2:
    pat ',' pat
  | patcn2 ',' pat
  ;

patrow:
    "..."
  | spatrow
  | patrow ',' spatrow
  ;
spatrow:
    LAB '=' pat
  | VID tyop
  | VID tyop "as" pat
  ;

pat:
    atpat
  | op VID atpat
  | pat VID pat
  | pat ':' ty
  | op VID tyop "as" pat
  ;

/* Type expressions */
ty:
    TYVAR
  | '{' tyrow '}'
  | tyseq TYCON
  | tysn2
  | ty "->" ty
  | '(' ty ')'
  ;
tyseq:
    ty
  | /* empty */
  | '(' tycn ')'
  | '(' ')'
  ;
tycn:
    ty
  | tycn ',' ty
  ;
tysn2:
    ty '*' ty
  | tysn2 '*' ty
  ;

tyrow:
    LAB ':' ty
  | tyrow ',' LAB ':' ty
  ;

%%
