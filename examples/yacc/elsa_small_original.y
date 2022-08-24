/* automatically generated grammar */
%glr-parser

%{
  static YYSTYPE IdExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE UnqualifiedIdMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQualifiedIdMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PostfixExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ExpressionListMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE UnaryExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE NewExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE NameAfterDotMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE NAD1Merge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE NAD2Merge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE BinExp_highMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE BinExp_midMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE BinaryExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ConditionalExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ExpressionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE StatementMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ConditionMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQTypeNameMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQTypeName_nccMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQTypeName_notfirstMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE InitDeclaratorMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQDtorNameMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PtrToMemberNameMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ParameterDeclarationMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ParameterDeclaratorMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ClassHeadNameOptMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE ClassHeadNameMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE PQClassNameMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE TemplateParameterListMerge (YYSTYPE L, YYSTYPE R);
  static YYSTYPE TemplateArgumentMerge (YYSTYPE L, YYSTYPE R);
%}
/* -------- tokens -------- */
%token TOK_EOF 0
%token TOK_NAME 1
%token TOK_INT_LITERAL 4
%token TOK_FLOAT_LITERAL 5
%token TOK_STRING_LITERAL 6
%token TOK_CHAR_LITERAL 7
%token TOK_AUTO 9
%token TOK_BOOL 11
%token TOK_CHAR 15
%token TOK_CONST 17
%token TOK_CONST_CAST 18
%token TOK_DELETE 21
%token TOK_DOUBLE 23
%token TOK_DYNAMIC_CAST 24
%token TOK_ENUM 26
%token TOK_EXTERN 29
%token TOK_FALSE 30
%token TOK_FLOAT 31
%token TOK_FRIEND 33
%token TOK_INLINE 36
%token TOK_INT 37
%token TOK_LONG 38
%token TOK_MUTABLE 39
%token TOK_NEW 41
%token TOK_OPERATOR 42
%token TOK_REGISTER 47
%token TOK_REINTERPRET_CAST 48
%token TOK_SHORT 50
%token TOK_SIGNED 51
%token TOK_SIZEOF 52
%token TOK_STATIC 53
%token TOK_STATIC_CAST 54
%token TOK_TEMPLATE 57
%token TOK_THIS 58
%token TOK_THROW 59
%token TOK_TRUE 60
%token TOK_TYPEDEF 62
%token TOK_TYPEID 63
%token TOK_TYPENAME 64
%token TOK_UNSIGNED 66
%token TOK_VIRTUAL 68
%token TOK_VOID 69
%token TOK_VOLATILE 70
%token TOK_WCHAR_T 71
%token TOK_LPAREN 73
%token TOK_RPAREN 74
%token TOK_LBRACKET 75
%token TOK_RBRACKET 76
%token TOK_ARROW 77
%token TOK_COLONCOLON 78
%token TOK_DOT 79
%token TOK_BANG 80
%token TOK_TILDE 81
%token TOK_PLUS 82
%token TOK_MINUS 83
%token TOK_PLUSPLUS 84
%token TOK_MINUSMINUS 85
%token TOK_AND 86
%token TOK_STAR 87
%token TOK_DOTSTAR 88
%token TOK_ARROWSTAR 89
%token TOK_SLASH 90
%token TOK_PERCENT 91
%token TOK_LEFTSHIFT 92
%token TOK_RIGHTSHIFT 93
%token TOK_LESSTHAN 94
%token TOK_LESSEQ 95
%token TOK_GREATERTHAN 96
%token TOK_GREATEREQ 97
%token TOK_EQUALEQUAL 98
%token TOK_NOTEQUAL 99
%token TOK_XOR 100
%token TOK_OR 101
%token TOK_ANDAND 102
%token TOK_OROR 103
%token TOK_QUESTION 104
%token TOK_COLON 105
%token TOK_EQUAL 106
%token TOK_STAREQUAL 107
%token TOK_SLASHEQUAL 108
%token TOK_PERCENTEQUAL 109
%token TOK_PLUSEQUAL 110
%token TOK_MINUSEQUAL 111
%token TOK_ANDEQUAL 112
%token TOK_XOREQUAL 113
%token TOK_OREQUAL 114
%token TOK_LEFTSHIFTEQUAL 115
%token TOK_RIGHTSHIFTEQUAL 116
%token TOK_COMMA 117
%token TOK_ELLIPSIS 118
%token TOK_SEMICOLON 119
%token TOK_LBRACE 120
%token TOK_RBRACE 121
%token TOK_PREFER_SHIFT 123


/* -------- precedence and associativity ---------*/
/* low precedence */
%nonassoc TOK_PREFER_SHIFT
%left TOK_OROR
%left TOK_ANDAND
%left TOK_OR
%left TOK_XOR
%left TOK_AND
%left TOK_EQUALEQUAL TOK_NOTEQUAL
%left TOK_LEFTSHIFT TOK_RIGHTSHIFT
%left TOK_PLUS TOK_MINUS
%left TOK_STAR TOK_SLASH TOK_PERCENT
%left TOK_DOTSTAR TOK_ARROWSTAR
%nonassoc TOK_CONST TOK_VOLATILE TOK_LBRACKET
%right TOK_COLONCOLON
/* high precedence */


/* -------- productions ------ */
%%

__EarlyStartSymbol: File TOK_EOF { $$=0; }
                  ;

File: TranslationUnit { $$=1; }
    ;

Identifier: TOK_NAME { $$=2; }
          ;

TranslationUnit: { $$=3; }
               | TranslationUnit Declaration { $$=4; }
               | TranslationUnit TOK_SEMICOLON { $$=5; }
               ;

PrimaryExpression: Literal { $$=6; }
                 | TOK_THIS { $$=7; }
                 | TOK_LPAREN Expression TOK_RPAREN { $$=8; }
                 | IdExpression { $$=9; }
                 ;

Literal: TOK_INT_LITERAL { $$=10; }
       | TOK_FLOAT_LITERAL { $$=11; }
       | StringLiteral { $$=12; }
       | TOK_CHAR_LITERAL { $$=13; }
       | TOK_TRUE { $$=14; }
       | TOK_FALSE { $$=15; }
       ;

PreprocString: TOK_STRING_LITERAL { $$=16; }
             ;

StringLiteral: PreprocString { $$=17; }
             | PreprocString StringLiteral { $$=18; }
             ;

IdExpression: PQualifiedId 
            | TOK_COLONCOLON PQualifiedId 
            ;

UnqualifiedId: Identifier 
             | OperatorFunctionId 
             | ConversionFunctionId 
             | TemplateId 
             ;

PQualifiedId: UnqualifiedId 
            | Identifier TOK_COLONCOLON PQualifiedId 
            | Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQualifiedId 
            | TOK_TEMPLATE Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQualifiedId 
            ;

ArgumentList: TOK_LPAREN ExpressionListOpt TOK_RPAREN { $$=29; }
            ;

PostfixExpression: PrimaryExpression 
                 | PostfixExpression TOK_LBRACKET Expression TOK_RBRACKET 
                 | PostfixExpression ArgumentList 
                 | TOK_TYPENAME IdExpression ArgumentList 
                 | CtorExpressionType ArgumentList 
                 | PostfixExpression TOK_DOT NameAfterDot 
                 | PostfixExpression TOK_ARROW NameAfterDot 
                 | PostfixExpression TOK_PLUSPLUS 
                 | PostfixExpression TOK_MINUSMINUS 
                 | CastKeyword TOK_LESSTHAN TypeId TOK_GREATERTHAN TOK_LPAREN Expression TOK_RPAREN 
                 | TOK_TYPEID TOK_LPAREN Expression TOK_RPAREN 
                 | TOK_TYPEID TOK_LPAREN TypeId TOK_RPAREN 
                 ;

CtorExpressionType: PQTypeName { $$=42; }
                  | TOK_CHAR { $$=43; }
                  | TOK_WCHAR_T { $$=44; }
                  | TOK_BOOL { $$=45; }
                  | TOK_SHORT { $$=46; }
                  | TOK_INT { $$=47; }
                  | TOK_LONG { $$=48; }
                  | TOK_SIGNED { $$=49; }
                  | TOK_UNSIGNED { $$=50; }
                  | TOK_FLOAT { $$=51; }
                  | TOK_DOUBLE { $$=52; }
                  | TOK_VOID { $$=53; }
                  ;

CastKeyword: TOK_DYNAMIC_CAST { $$=54; }
           | TOK_STATIC_CAST { $$=55; }
           | TOK_REINTERPRET_CAST { $$=56; }
           | TOK_CONST_CAST { $$=57; }
           ;

ExpressionList: AssignmentExpression 
              | AssignmentExpression TOK_COMMA ExpressionList 
              ;

ExpressionListOpt: { $$=60; }
                 | ExpressionList { $$=61; }
                 ;

UnaryExpression: PostfixExpression 
               | TOK_PLUSPLUS UnaryExpression 
               | TOK_MINUSMINUS UnaryExpression 
               | TOK_SIZEOF UnaryExpression 
                                                                                       | DeleteExpression 
               | TOK_STAR UnaryExpression 
               | TOK_AND UnaryExpression 
               | TOK_PLUS UnaryExpression 
               | TOK_MINUS UnaryExpression 
               | TOK_BANG UnaryExpression 
               | TOK_TILDE UnaryExpression 
               | TOK_SIZEOF TOK_LPAREN TypeId TOK_RPAREN 
               ;
/*
ColonColonOpt: { $$=75; }
             | TOK_COLONCOLON 
             ;

NewExpression: ColonColonOpt TOK_NEW NewPlacementOpt NewTypeId NewInitializerOpt 
             | ColonColonOpt TOK_NEW NewPlacementOpt TOK_LPAREN TypeId TOK_RPAREN NewInitializerOpt 
             ;

NewPlacementOpt: { $$=79; }
               | TOK_LPAREN ExpressionList TOK_RPAREN { $$=80; }
               ;

NewTypeId: TypeSpecifier NewDeclaratorOpt { $$=81; }
         ;

NewDeclaratorOpt: { $$=82; }
                | TOK_STAR CVQualifierSeqOpt NewDeclaratorOpt 
                | PtrToMemberName TOK_STAR CVQualifierSeqOpt NewDeclaratorOpt 
                | DirectNewDeclarator { $$=85; }
                ;

DirectNewDeclarator: TOK_LBRACKET Expression TOK_RBRACKET { $$=86; }
                   | DirectNewDeclarator TOK_LBRACKET ConstantExpression TOK_RBRACKET { $$=87; }
                   ;

NewInitializerOpt: { $$=88; }
                 | TOK_LPAREN ExpressionListOpt TOK_RPAREN { $$=89; }
                 ;

DeleteExpression: ColonColonOpt TOK_DELETE UnaryExpression { $$=90; }
                | ColonColonOpt TOK_DELETE TOK_LBRACKET TOK_RBRACKET UnaryExpression { $$=91; }
                ;*/

NameAfterDot: NAD1 
            | TOK_COLONCOLON NAD2 
            ;

NAD1: NAD2 
    | TOK_TEMPLATE Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN 
    | TOK_TILDE Identifier 
    | TOK_TILDE Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN 
    | ConversionFunctionId 
    | TOK_TEMPLATE Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN TOK_COLONCOLON NAD1 
    ;

NAD2: Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN 
    | Identifier 
    | OperatorFunctionId 
    | Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN TOK_COLONCOLON NAD1 
    | Identifier TOK_COLONCOLON NAD1 
    ;
/*
BinExp_high: UnaryExpression 
           | BinExp_high TOK_DOTSTAR BinExp_high 
           | BinExp_high TOK_ARROWSTAR BinExp_high 
           | BinExp_high TOK_STAR BinExp_high 
           | BinExp_high TOK_SLASH BinExp_high 
           | BinExp_high TOK_PERCENT BinExp_high 
           | BinExp_high TOK_PLUS BinExp_high 
           | BinExp_high TOK_MINUS BinExp_high 
           | BinExp_high TOK_LEFTSHIFT BinExp_high 
           | BinExp_high TOK_RIGHTSHIFT BinExp_high 
           ;

BinExp_mid: BinExp_high 
          | BinExp_mid TOK_LESSTHAN BinExp_high 
          | BinExp_mid TOK_GREATERTHAN BinExp_high 
          | BinExp_mid TOK_LESSEQ BinExp_high 
          | BinExp_mid TOK_GREATEREQ BinExp_high 
          ;
*/
BinaryExpression: /*BinExp_mid*/ UnaryExpression 
                | BinaryExpression TOK_EQUALEQUAL BinaryExpression 
                | BinaryExpression TOK_NOTEQUAL BinaryExpression 
                | BinaryExpression TOK_AND BinaryExpression 
                | BinaryExpression TOK_XOR BinaryExpression 
                | BinaryExpression TOK_OR BinaryExpression 
                | BinaryExpression TOK_ANDAND BinaryExpression 
                | BinaryExpression TOK_OROR BinaryExpression 
                ;

ConditionalExpression: BinaryExpression 
                     | BinaryExpression TOK_QUESTION Expression TOK_COLON AssignmentExpression 
                     ;

AssignmentExpression: ConditionalExpression { $$=132; }
                    | BinaryExpression AssignmentOperator AssignmentExpression { $$=133; }
                    | ThrowExpression { $$=134; }
                    ;

AssignmentOperator: TOK_STAREQUAL { $$=135; }
                  | TOK_SLASHEQUAL { $$=136; }
                  | TOK_PERCENTEQUAL { $$=137; }
                  | TOK_PLUSEQUAL { $$=138; }
                  | TOK_MINUSEQUAL { $$=139; }
                  | TOK_RIGHTSHIFTEQUAL { $$=140; }
                  | TOK_LEFTSHIFTEQUAL { $$=141; }
                  | TOK_ANDEQUAL { $$=142; }
                  | TOK_XOREQUAL { $$=143; }
                  | TOK_OREQUAL { $$=144; }
                  | TOK_EQUAL { $$=145; }
                  ;

Expression: AssignmentExpression 
          | Expression TOK_COMMA AssignmentExpression 
          ;

ConstantExpression: AssignmentExpression { $$=150; }
                  ;

ConstantExpressionOpt: { $$=151; }
                     | ConstantExpression { $$=152; }
                     ;

Declaration: BlockDeclaration { $$=185; }
           ;

BlockDeclaration: SimpleDeclaration { $$=193; }
                ;

SimpleDeclaration: DeclSpecifier InitDeclaratorList TOK_SEMICOLON { $$=194; }
                 | DeclSpecifier TOK_SEMICOLON { $$=195; }
                 ;

DeclSpecifier: PQTypeName UberModifierSeqOpt { $$=196; }
             | UberModifierSeq PQTypeName UberModifierSeqOpt { $$=197; }
             | UberTypeKeyword UberTypeAndModifierSeqOpt { $$=198; }
             | UberModifierSeq UberTypeKeyword UberTypeAndModifierSeqOpt { $$=199; }
             | ElaboratedOrSpecifier UberModifierSeqOpt { $$=200; }
             | UberModifierSeq ElaboratedOrSpecifier UberModifierSeqOpt { $$=201; }
             ;

ElaboratedOrSpecifier: ElaboratedTypeSpecifier { $$=202; }
                     | EnumSpecifier { $$=204; }
                     ;

UberModifierSeq: UberModifier { $$=205; }
               | UberModifierSeq UberModifier { $$=206; }
               ;

UberModifierSeqOpt: { $$=207; }
                  | UberModifierSeq { $$=208; }
                  ;

UberTypeAndModifierSeqOpt: { $$=209; }
                         | UberTypeAndModifierSeqOpt UberModifier { $$=210; }
                         | UberTypeAndModifierSeqOpt UberTypeKeyword { $$=211; }
                         ;

UberCVQualifierSeq: UberCVQualifier { $$=212; }
                  | UberCVQualifierSeq UberCVQualifier { $$=213; }
                  ;

UberCVQualifierSeqOpt: { $$=214; }
                     | UberCVQualifierSeq { $$=215; }
                     ;

UberTypeAndCVQualifierSeqOpt: { $$=216; }
                            | UberTypeAndCVQualifierSeqOpt UberCVQualifier { $$=217; }
                            | UberTypeAndCVQualifierSeqOpt UberTypeKeyword { $$=218; }
                            ;

UberModifier: TOK_AUTO { $$=219; }
            | TOK_REGISTER { $$=220; }
            | TOK_STATIC { $$=221; }
            | TOK_EXTERN { $$=222; }
            | TOK_MUTABLE { $$=223; }
            | TOK_INLINE { $$=224; }
            | TOK_VIRTUAL { $$=225; }
            | TOK_FRIEND { $$=226; }
            | TOK_TYPEDEF { $$=227; }
            | TOK_CONST 
            | TOK_VOLATILE 
            ;

UberCVQualifier: TOK_CONST 
               | TOK_VOLATILE 
               ;

UberTypeKeyword: TOK_CHAR { $$=232; }
               | TOK_WCHAR_T { $$=233; }
               | TOK_BOOL { $$=234; }
               | TOK_SHORT { $$=235; }
               | TOK_INT { $$=236; }
               | TOK_LONG { $$=237; }
               | TOK_SIGNED { $$=238; }
               | TOK_UNSIGNED { $$=239; }
               | TOK_FLOAT { $$=240; }
               | TOK_DOUBLE { $$=241; }
               | TOK_VOID { $$=242; }
               ;

ElaboratedTypeSpecifier: TOK_TYPENAME PQTypeName { $$=245; }
                       ;

TypeSpecifier: PQTypeName UberCVQualifierSeqOpt { $$=246; }
             | UberCVQualifierSeq PQTypeName UberCVQualifierSeqOpt { $$=247; }
             | UberTypeKeyword UberTypeAndCVQualifierSeqOpt { $$=248; }
             | UberCVQualifierSeq UberTypeKeyword UberTypeAndCVQualifierSeqOpt { $$=249; }
             | ElaboratedOrSpecifier UberCVQualifierSeqOpt { $$=250; }
             | UberCVQualifierSeq ElaboratedOrSpecifier UberCVQualifierSeqOpt { $$=251; }
             ;

PQTypeName: PQTypeName_ncc 
          | TOK_COLONCOLON PQTypeName_ncc 
          ;

PQTypeName_ncc: Identifier 
              | TemplateId 
              | Identifier TOK_COLONCOLON PQTypeName_notfirst 
              | Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQTypeName_notfirst 
              ;

PQTypeName_notfirst: PQTypeName_ncc 
                   | TOK_TEMPLATE TemplateId 
                   | TOK_TEMPLATE Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQTypeName_notfirst 
                   ;

EnumSpecifier: TOK_ENUM TOK_LBRACE EnumeratorListOpt TOK_RBRACE { $$=261; }
             | TOK_ENUM Identifier TOK_LBRACE EnumeratorListOpt TOK_RBRACE { $$=262; }
             ;

EnumeratorListOpt: { $$=263; }
                 | EnumeratorDefinition { $$=264; }
                 | EnumeratorDefinition TOK_COMMA EnumeratorListOpt { $$=265; }
                 ;

EnumeratorDefinition: Identifier { $$=266; }
                    | Identifier TOK_EQUAL ConstantExpression { $$=267; }
                    ;

InitDeclaratorList: InitDeclarator { $$=271; }
                  | InitDeclarator TOK_COMMA InitDeclaratorList { $$=272; }
                  ;

InitDeclarator: Declarator 
              | Declarator Initializer 
              ;

Initializer: TOK_EQUAL SimpleInitializerClause { $$=275; }
           | TOK_LPAREN ExpressionList TOK_RPAREN { $$=276; }
           ;

SimpleInitializerClause: AssignmentExpression { $$=277; }
                       | CompoundInitializer { $$=278; }
                       ;

InitializerClause: SimpleInitializerClause { $$=279; }
                 ;

CompoundInitializer: TOK_LBRACE InitializerList CommaOpt TOK_RBRACE { $$=280; }
                   | TOK_LBRACE TOK_RBRACE { $$=281; }
                   ;

CommaOpt: { $$=282; }
        | TOK_COMMA { $$=283; }
        ;

InitializerList: InitializerClause { $$=284; }
               | InitializerList TOK_COMMA InitializerClause { $$=285; }
               ;

Declarator: TOK_STAR CVQualifierSeqOpt Declarator 
          | TOK_AND Declarator 
          | PtrToMemberName TOK_STAR CVQualifierSeqOpt Declarator 
          | DirectDeclarator { $$=289; }
          ;

DirectDeclarator: IdExpression { $$=290; }
                | PQDtorName { $$=291; }
                | DirectDeclarator TOK_LPAREN ParameterDeclarationClause TOK_RPAREN CVQualifierSeqOpt ExceptionSpecificationOpt { $$=292; }
                | DirectDeclarator TOK_LBRACKET ConstantExpressionOpt TOK_RBRACKET { $$=293; }
                | TOK_LPAREN Declarator TOK_RPAREN { $$=294; }
                ;

PQDtorName: TOK_TILDE Identifier 
          | TOK_TILDE Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN 
          | Identifier TOK_COLONCOLON PQDtorName 
          | Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQDtorName 
          | TOK_TEMPLATE Identifier TOK_LESSTHAN TemplateArgumentList TOK_GREATERTHAN TOK_COLONCOLON PQDtorName 
          ;

PtrToMemberName: IdExpression TOK_COLONCOLON 
               ;

CVQualifierSeqOpt: { $$=301; }
                 | CVQualifierSeq { $$=302; }
                 ;

CVQualifierSeq: CVQualifier { $$=303; }
              | CVQualifier CVQualifierSeq { $$=304; }
              ;

CVQualifier: TOK_CONST 
           | TOK_VOLATILE 
           ;

TypeId: TypeSpecifier AbstractDeclaratorOpt { $$=307; }
      ;

AbstractDeclaratorOpt: { $$=308; }
                     | AbstractDeclarator { $$=309; }
                     ;

AbstractDeclarator: TOK_STAR CVQualifierSeqOpt AbstractDeclaratorOpt 
                  | TOK_AND AbstractDeclaratorOpt 
                  | PtrToMemberName TOK_STAR CVQualifierSeqOpt AbstractDeclaratorOpt 
                  | DirectAbstractDeclarator { $$=313; }
                  ;

DirectAbstractDeclaratorOpt: { $$=314; }
                           | DirectAbstractDeclarator { $$=315; }
                           ;

DirectAbstractDeclarator: DirectAbstractDeclaratorOpt TOK_LPAREN ParameterDeclarationClause TOK_RPAREN CVQualifierSeqOpt ExceptionSpecificationOpt { $$=316; }
                        | DirectAbstractDeclaratorOpt TOK_LBRACKET ConstantExpressionOpt TOK_RBRACKET { $$=317; }
                        | TOK_LPAREN AbstractDeclarator TOK_RPAREN { $$=318; }
                        ;

ParameterDeclarationClause: ParameterDeclarationList { $$=319; }
                          | { $$=320; }
                          ;

ParameterDeclarationList: TOK_ELLIPSIS { $$=321; }
                        | ParameterDeclaration TOK_ELLIPSIS { $$=322; }
                        | ParameterDeclaration { $$=323; }
                        | ParameterDeclaration TOK_COMMA ParameterDeclarationList { $$=324; }
                        ;

ParameterDeclaration: TypeSpecifier ParameterDeclarator 
                    | TOK_REGISTER TypeSpecifier ParameterDeclarator 
                    | TypeSpecifier TOK_REGISTER ParameterDeclarator 
                    ;

ParameterDeclarator: UnqualifiedDeclarator 
                   | UnqualifiedDeclarator TOK_EQUAL AssignmentExpression 
                   | AbstractDeclaratorOpt 
                   | AbstractDeclaratorOpt TOK_EQUAL AssignmentExpression 
                   ;

ConversionFunctionId: TOK_OPERATOR ConversionTypeId { $$=394; }
                    ;

ConversionTypeId: TypeSpecifier ConversionDeclaratorOpt { $$=395; }
                ;

ConversionDeclaratorOpt: 
                       | TOK_STAR CVQualifierSeqOpt ConversionDeclaratorOpt 
                       | TOK_AND ConversionDeclaratorOpt 
                       | PtrToMemberName TOK_STAR CVQualifierSeqOpt ConversionDeclaratorOpt 
                       ;

OperatorFunctionId: TOK_OPERATOR Operator { $$=404; }
                  ;

Operator: TOK_NEW 
        | TOK_DELETE 
        | TOK_NEW TOK_LBRACKET TOK_RBRACKET { $$=407; }
        | TOK_DELETE TOK_LBRACKET TOK_RBRACKET { $$=408; }
        | TOK_BANG { $$=409; }
        | TOK_TILDE { $$=410; }
        | TOK_PLUSPLUS { $$=411; }
        | TOK_MINUSMINUS { $$=412; }
        | TOK_PLUS 
        | TOK_MINUS 
        | TOK_STAR 
        | TOK_SLASH 
        | TOK_PERCENT 
        | TOK_LEFTSHIFT 
        | TOK_RIGHTSHIFT 
        | TOK_AND 
        | TOK_XOR 
        | TOK_OR 
        | TOK_EQUAL { $$=423; }
        | TOK_PLUSEQUAL { $$=424; }
        | TOK_MINUSEQUAL { $$=425; }
        | TOK_STAREQUAL { $$=426; }
        | TOK_SLASHEQUAL { $$=427; }
        | TOK_PERCENTEQUAL { $$=428; }
        | TOK_LEFTSHIFTEQUAL { $$=429; }
        | TOK_RIGHTSHIFTEQUAL { $$=430; }
        | TOK_ANDEQUAL { $$=431; }
        | TOK_XOREQUAL { $$=432; }
        | TOK_OREQUAL { $$=433; }
        | TOK_EQUALEQUAL 
        | TOK_NOTEQUAL 
        | TOK_LESSTHAN { $$=436; }
        | TOK_GREATERTHAN { $$=437; }
        | TOK_LESSEQ { $$=438; }
        | TOK_GREATEREQ { $$=439; }
        | TOK_ANDAND 
        | TOK_OROR 
        | TOK_ARROW { $$=442; }
        | TOK_ARROWSTAR 
        | TOK_LBRACKET TOK_RBRACKET { $$=444; }
        | TOK_LPAREN TOK_RPAREN { $$=445; }
        | TOK_COMMA { $$=446; }
        ;

TemplateArgumentListOpt: { $$=463; }
                       | TemplateArgumentList { $$=464; }
                       ;

TemplateId: Identifier TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN { $$=465; }
          | OperatorFunctionId TOK_LESSTHAN TemplateArgumentListOpt TOK_GREATERTHAN { $$=466; }
          ;

TemplateArgumentList: TemplateArgument { $$=467; }
                    ;

TemplateArgumentListTailOpt: { $$=468; }
                           | TOK_COMMA TemplateArgument { $$=469; }
                           ;

TemplateArgument: TypeId TemplateArgumentListTailOpt 
                | AssignmentExpression TemplateArgumentListTailOpt 
                ;

UnqualifiedDeclarator: Declarator { $$=480; }
                     ;

ThrowExpression: TOK_THROW { $$=481; }
               | TOK_THROW AssignmentExpression { $$=482; }
               ;

ExceptionSpecificationOpt: { $$=483; }
                         | TOK_THROW TOK_LPAREN TOK_RPAREN { $$=484; }
                         | TOK_THROW TOK_LPAREN TypeIdList TOK_RPAREN { $$=485; }
                         ;

TypeIdList: TypeId { $$=486; }
          | TypeId TOK_COMMA TypeIdList { $$=487; }
          ;

%%

static YYSTYPE IdExpressionMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE UnqualifiedIdMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PQualifiedIdMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PostfixExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ExpressionListMerge (YYSTYPE L, YYSTYPE R)
{  L->first()->addAmbiguity(R->first()); return L; }

static YYSTYPE UnaryExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE NewExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE NameAfterDotMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE NAD1Merge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE NAD2Merge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE BinExp_highMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE BinExp_midMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE BinaryExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ConditionalExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ExpressionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE StatementMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ConditionMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE PQTypeNameMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PQTypeName_nccMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PQTypeName_notfirstMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE InitDeclaratorMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE PQDtorNameMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PtrToMemberNameMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE ParameterDeclarationMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ParameterDeclaratorMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE ClassHeadNameOptMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE ClassHeadNameMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE PQClassNameMerge (YYSTYPE L, YYSTYPE R)
{  return L->mergeAmbiguous(R); }

static YYSTYPE TemplateParameterListMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

static YYSTYPE TemplateArgumentMerge (YYSTYPE L, YYSTYPE R)
{  L->addAmbiguity(R); return L; }

