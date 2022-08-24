/*	$Id: parser.y,v 1.3 1997/11/19 15:13:16 sandro Exp $	*/

/*
 * Copyright (c) 1997 Sandro Sigala <ssigala@globalnet.it>.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
 * IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 * NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

/*
 * ISO C++ parser.
 *
 * Based on the ISO C++ draft standard of December '96.
 */

%{
#include <stdio.h>

extern int lineno;

static void yyerror(char *s);
%}

%token IDENTIFIER INTEGER FLOATING CHARACTER STRING
%token TYPEDEF_NAME NAMESPACE_NAME CLASS_NAME ENUM_NAME TEMPLATE_NAME

%token ELLIPSIS COLONCOLON DOTSTAR ADDEQ SUBEQ MULEQ DIVEQ MODEQ
%token XOREQ ANDEQ OREQ SL SR SREQ SLEQ EQ NOTEQ LTEQ GTEQ ANDAND OROR
%token PLUSPLUS MINUSMINUS ARROWSTAR ARROW

%token ASM AUTO BOOL BREAK CASE CATCH CHAR CLASS CONST CONST_CAST CONTINUE
%token DEFAULT DELETE DO DOUBLE DYNAMIC_CAST ELSE ENUM EXPLICIT EXPORT EXTERN
%token FALSE FLOAT FOR FRIEND GOTO IF INLINE INT LONG MUTABLE NAMESPACE NEW
%token OPERATOR PRIVATE PROTECTED PUBLIC REGISTER REINTERPRET_CAST RETURN
%token SHORT SIGNED SIZEOF STATIC STATIC_CAST STRUCT SWITCH TEMPLATE THIS
%token THROW TRUE TRY TYPEDEF TYPEID TYPENAME UNION UNSIGNED USING VIRTUAL
%token VOID VOLATILE WCHAR_T WHILE

%start translation_unit

%%

/*----------------------------------------------------------------------
 * Context-dependent identifiers.
 *----------------------------------------------------------------------*/

typedef_name:
	/* identifier */
	TYPEDEF_NAME
	;

namespace_name:
	original_namespace_name
	| namespace_alias
	;

original_namespace_name:
	/* identifier */
	NAMESPACE_NAME
	;

namespace_alias:
	/* identifier */
	NAMESPACE_NAME
	;

class_name:
	/* identifier */
	CLASS_NAME
	| template_id
	;

enum_name:
	/* identifier */
	ENUM_NAME
	;

template_name:
	/* identifier */
	TEMPLATE_NAME
	;

/*----------------------------------------------------------------------
 * Lexical elements.
 *----------------------------------------------------------------------*/

identifier:
	IDENTIFIER
	;

literal:
	integer_literal
	| character_literal
	| floating_literal
	| string_literal
	| boolean_literal
	;

integer_literal:
	INTEGER
	;

character_literal:
	CHARACTER
	;

floating_literal:
	FLOATING
	;

string_literal:
	STRING
	;

boolean_literal:
	TRUE
	| FALSE
	;

/*----------------------------------------------------------------------
 * Translation unit.
 *----------------------------------------------------------------------*/

translation_unit:
	declaration_seq_opt
	;

/*----------------------------------------------------------------------
 * Expressions.
 *----------------------------------------------------------------------*/

primary_expression:
	literal
	| THIS
	| COLONCOLON identifier
	| COLONCOLON operator_function_id
	| COLONCOLON qualified_id
	| '(' expression ')'
	| id_expression
	;

id_expression:
	unqualified_id
	| qualified_id
	;

unqualified_id:
	identifier
	| operator_function_id
	| conversion_function_id
	| '~' class_name
	| template_id
	;

qualified_id:
	nested_name_specifier TEMPLATE_opt unqualified_id
	;

nested_name_specifier:
	class_or_namespace_name COLONCOLON nested_name_specifier_opt
	;

class_or_namespace_name:
	class_name
	| namespace_name
	;

postfix_expression:
	primary_expression
	| postfix_expression '[' expression ']'
	| postfix_expression '(' expression_list_opt ')'
	| simple_type_specifier '(' expression_list_opt ')'
	| postfix_expression '.' TEMPLATE_opt COLONCOLON_opt id_expression
	| postfix_expression ARROW TEMPLATE_opt COLONCOLON_opt id_expression
	| postfix_expression '.' pseudo_destructor_name
	| postfix_expression ARROW pseudo_destructor_name
	| postfix_expression PLUSPLUS
	| postfix_expression MINUSMINUS
	| DYNAMIC_CAST '<' type_id '>' '(' expression ')'
	| STATIC_CAST '<' type_id '>' '(' expression ')'
	| REINTERPRET_CAST '<' type_id '>' '(' expression ')'
	| CONST_CAST '<' type_id '>' '(' expression ')'
	| TYPEID '(' expression ')'
	| TYPEID '(' type_id ')'
	;

expression_list:
	assignment_expression
	| expression_list ',' assignment_expression
	;

pseudo_destructor_name:
	COLONCOLON_opt nested_name_specifier_opt type_name COLONCOLON '~' type_name
	| COLONCOLON_opt nested_name_specifier_opt '~' type_name
	;

unary_expression:
	postfix_expression
	| PLUSPLUS cast_expression
	| MINUSMINUS cast_expression
	| unary_operator cast_expression
	| SIZEOF unary_expression
	| SIZEOF '(' type_id ')'
	| new_expression
	| delete_expression
	;

unary_operator:
	'*'
	| '&'
	| '+'
	| '-'
	| '!'
	| '~'
	;

new_expression:
	COLONCOLON_opt NEW new_placement_opt new_type_id new_initializer_opt
	| COLONCOLON_opt NEW new_placement_opt '(' type_id ')' new_initializer_opt
	;

new_placement:
	'(' expression_list ')'
	;

new_type_id:
	type_specifier_seq new_declarator_opt
	;

new_declarator:
	ptr_operator new_declarator_opt
	| direct_new_declarator
	;

direct_new_declarator:
	'[' expression ']'
	| direct_new_declarator '[' constant_expression ']'
	;

new_initializer:
	'(' expression_list_opt ')'
	;

delete_expression:
	COLONCOLON_opt DELETE cast_expression
	| COLONCOLON_opt DELETE '[' ']' cast_expression
	;

cast_expression:
	unary_expression
	| '(' type_id ')' cast_expression
	;

pm_expression:
	cast_expression
	| pm_expression DOTSTAR cast_expression
	| pm_expression ARROWSTAR cast_expression
	;

multiplicative_expression:
	pm_expression
	| multiplicative_expression '*' pm_expression
	| multiplicative_expression '/' pm_expression
	| multiplicative_expression '%' pm_expression
	;

additive_expression:
	multiplicative_expression
	| additive_expression '+' multiplicative_expression
	| additive_expression '-' multiplicative_expression
	;

shift_expression:
	additive_expression
	| shift_expression SL additive_expression
	| shift_expression SR additive_expression
	;

relational_expression:
	shift_expression
	| relational_expression '<' shift_expression
	| relational_expression '>' shift_expression
	| relational_expression LTEQ shift_expression
	| relational_expression GTEQ shift_expression
	;

equality_expression:
	relational_expression
	| equality_expression EQ relational_expression
	| equality_expression NOTEQ relational_expression
	;

and_expression:
	equality_expression
	| and_expression '&' equality_expression
	;

exclusive_or_expression:
	and_expression
	| exclusive_or_expression '^' and_expression
	;

inclusive_or_expression:
	exclusive_or_expression
	| inclusive_or_expression '|' exclusive_or_expression
	;

logical_and_expression:
	inclusive_or_expression
	| logical_and_expression ANDAND inclusive_or_expression
	;

logical_or_expression:
	logical_and_expression
	| logical_or_expression OROR logical_and_expression
	;

conditional_expression:
	logical_or_expression
	| logical_or_expression  '?' expression ':' assignment_expression
	;

assignment_expression:
	conditional_expression
	| logical_or_expression assignment_operator assignment_expression
	| throw_expression
	;

assignment_operator:
	'='
	| MULEQ
	| DIVEQ
	| MODEQ
	| ADDEQ
	| SUBEQ
	| SREQ
	| SLEQ
	| ANDEQ
	| XOREQ
	| OREQ
	;

expression:
	assignment_expression
	| expression ',' assignment_expression
	;

constant_expression:
	conditional_expression
	;

/*----------------------------------------------------------------------
 * Statements.
 *----------------------------------------------------------------------*/

statement:
	labeled_statement
	| expression_statement
	| compound_statement
	| selection_statement
	| iteration_statement
	| jump_statement
	| declaration_statement
	| try_block
	;

labeled_statement:
	identifier ':' statement
	| CASE constant_expression ':' statement
	| DEFAULT ':' statement
	;

expression_statement:
	expression_opt ';'
	;

compound_statement:
	'{' statement_seq_opt '}'
	;

statement_seq:
	statement
	| statement_seq statement
	;

selection_statement:
	IF '(' condition ')' statement
	| IF '(' condition ')' statement ELSE statement
	| SWITCH '(' condition ')' statement
	;

condition:
	expression
	| type_specifier_seq declarator '=' assignment_expression
	;

iteration_statement:
	WHILE '(' condition ')' statement
	| DO statement WHILE '(' expression ')' ';'
	| FOR '(' for_init_statement condition_opt ';' expression_opt ')' statement
	;

for_init_statement:
	expression_statement
	| simple_declaration
	;

jump_statement:
	BREAK ';'
	| CONTINUE ';'
	| RETURN expression_opt ';'
	| GOTO identifier ';'
	;

declaration_statement:
	block_declaration
	;

/*----------------------------------------------------------------------
 * Declarations.
 *----------------------------------------------------------------------*/

declaration_seq:
	declaration
	| declaration_seq declaration
	;

declaration:
	block_declaration
	| function_definition
	| template_declaration
	| explicit_instantiation
	| explicit_specialization
	| linkage_specification
	| namespace_definition
	;

block_declaration:
	simple_declaration
	| asm_definition
	| namespace_alias_definition
	| using_declaration
	| using_directive
	;

simple_declaration:
	decl_specifier_seq_opt init_declarator_list_opt ';'
	;

decl_specifier:
	storage_class_specifier
	| type_specifier
	| function_specifier
	| FRIEND
	| TYPEDEF
	;

decl_specifier_seq:
	decl_specifier_seq_opt decl_specifier
	;

storage_class_specifier:
	AUTO
	| REGISTER
	| STATIC
	| EXTERN
	| MUTABLE
	;

function_specifier:
	INLINE
	| VIRTUAL
	| EXPLICIT
	;

/* typedef_name: identifier ; */

type_specifier:
	simple_type_specifier
	| class_specifier
	| enum_specifier
	| elaborated_type_specifier
	| cv_qualifier
	;

simple_type_specifier:
	COLONCOLON_opt nested_name_specifier_opt type_name
	| CHAR
	| WCHAR_T
	| BOOL
	| SHORT
	| INT
	| LONG
	| SIGNED
	| UNSIGNED
	| FLOAT
	| DOUBLE
	| VOID
	;

type_name:
	class_name
	| enum_name
	| typedef_name
	;

elaborated_type_specifier:
	class_key COLONCOLON_opt nested_name_specifier_opt identifier
	| ENUM COLONCOLON_opt nested_name_specifier_opt identifier
	| TYPENAME COLONCOLON_opt nested_name_specifier identifier
	| TYPENAME COLONCOLON_opt nested_name_specifier identifier '<' template_argument_list '>'
	;

/* enum_name: identifier ; */

enum_specifier:
	ENUM identifier_opt '{' enumerator_list_opt '}'
	;

enumerator_list:
	enumerator_definition
	| enumerator_list ',' enumerator_definition
	;

enumerator_definition:
	enumerator
	| enumerator '=' constant_expression
	;

enumerator:
	identifier
	;

/* namespace_name: original_namespace_name | namespace_alias ; */
/* original_namespace_name: identifier ; */

namespace_definition:
	named_namespace_definition
	| unnamed_namespace_definition
	;

named_namespace_definition:
	original_namespace_definition
	| extension_namespace_definition
	;

original_namespace_definition:
	NAMESPACE identifier '{' namespace_body '}'
	;

extension_namespace_definition:
	NAMESPACE original_namespace_name '{' namespace_body '}'
	;

unnamed_namespace_definition:
	NAMESPACE '{' namespace_body '}'
	;

namespace_body:
	declaration_seq_opt
	;

/* namespace_alias: identifier ; */

namespace_alias_definition:
	NAMESPACE identifier '=' qualified_namespace_specifier ';'
	;

qualified_namespace_specifier:
	COLONCOLON_opt nested_name_specifier_opt namespace_name
	;

using_declaration:
	USING TYPENAME_opt COLONCOLON_opt nested_name_specifier unqualified_id ';'
	| USING COLONCOLON unqualified_id ';'
	;

using_directive:
	USING NAMESPACE COLONCOLON_opt nested_name_specifier_opt namespace_name ';'
	;

asm_definition:
	ASM '(' string_literal ')' ';'
	;

linkage_specification:
	EXTERN string_literal '{' declaration_seq_opt '}'
	| EXTERN string_literal declaration
	;

/*----------------------------------------------------------------------
 * Declarators.
 *----------------------------------------------------------------------*/

init_declarator_list:
	init_declarator
	| init_declarator_list ',' init_declarator
	;

init_declarator:
	declarator initializer_opt
	;

declarator:
	direct_declarator
	| ptr_operator declarator
	;

direct_declarator:
	declarator_id
	| direct_declarator '('parameter_declaration_clause ')' cv_qualifier_seq_opt exception_specification_opt
	| direct_declarator '[' constant_expression_opt ']'
	| '(' declarator ')'
	;

ptr_operator:
	'*' cv_qualifier_seq_opt
	| '&'
	| COLONCOLON_opt nested_name_specifier '*' cv_qualifier_seq_opt
	;

cv_qualifier_seq:
	cv_qualifier cv_qualifier_seq_opt
	;

cv_qualifier:
	CONST
	| VOLATILE
	;

declarator_id:
	COLONCOLON_opt id_expression
	| COLONCOLON_opt nested_name_specifier_opt type_name
	;

type_id:
	type_specifier_seq abstract_declarator_opt
	;

type_specifier_seq:
	type_specifier type_specifier_seq_opt
	;

abstract_declarator:
	ptr_operator abstract_declarator_opt
	| direct_abstract_declarator
	;

direct_abstract_declarator:
	direct_abstract_declarator_opt '(' parameter_declaration_clause ')' cv_qualifier_seq_opt exception_specification_opt
	| direct_abstract_declarator_opt '[' constant_expression_opt ']'
	| '(' abstract_declarator ')'
	;

parameter_declaration_clause:
	parameter_declaration_list_opt ELLIPSIS_opt
	| parameter_declaration_list ',' ELLIPSIS
	;

parameter_declaration_list:
	parameter_declaration
	| parameter_declaration_list ',' parameter_declaration
	;

parameter_declaration:
	decl_specifier_seq declarator
	| decl_specifier_seq declarator '=' assignment_expression
	| decl_specifier_seq abstract_declarator_opt
	| decl_specifier_seq abstract_declarator_opt '=' assignment_expression
	;

function_definition:
	decl_specifier_seq_opt declarator ctor_initializer_opt function_body
	| decl_specifier_seq_opt declarator function_try_block
	;

function_body:
	compound_statement
	;

initializer:
	'=' initializer_clause
	| '(' expression_list ')'
	;

initializer_clause:
	assignment_expression
	| '{' initializer_list COMMA_opt '}'
	| '{' '}'
	;

initializer_list:
	initializer_clause
	| initializer_list ',' initializer_clause
	;

/*----------------------------------------------------------------------
 * Classes.
 *----------------------------------------------------------------------*/

/* class_name: identifier | template_id ; */

class_specifier:
	class_head '{' member_specification_opt '}'
	;

class_head:
	class_key identifier_opt base_clause_opt
	| class_key nested_name_specifier identifier base_clause_opt
	;

class_key:
	CLASS
	| STRUCT
	| UNION
	;

member_specification:
	member_declaration member_specification_opt
	| access_specifier ':' member_specification_opt
	;

member_declaration:
	decl_specifier_seq_opt member_declarator_list_opt ';'
	| function_definition SEMICOLON_opt
	| qualified_id ';'
	| using_declaration
	| template_declaration
	;

member_declarator_list:
	member_declarator
	| member_declarator_list ',' member_declarator
	;

member_declarator:
	declarator pure_specifier_opt
	| declarator constant_initializer_opt
	| identifier_opt ':' constant_expression
	;

/*
 * This rule need a hack for working around the ``= 0'' pure specifier.
 * 0 is returned as an ``INTEGER'' by the lexical analyzer but in this
 * context is different.
 */
pure_specifier:
	'=' '0'
	;

constant_initializer:
	'=' constant_expression
	;

/*----------------------------------------------------------------------
 * Derived classes.
 *----------------------------------------------------------------------*/

base_clause:
	':' base_specifier_list
	;

base_specifier_list:
	base_specifier
	| base_specifier_list ',' base_specifier
	;

base_specifier:
	COLONCOLON_opt nested_name_specifier_opt class_name
	| VIRTUAL access_specifier_opt COLONCOLON_opt nested_name_specifier_opt class_name
	| access_specifier VIRTUAL_opt COLONCOLON_opt nested_name_specifier_opt class_name
	;

access_specifier:
	PRIVATE
	| PROTECTED
	| PUBLIC
	;

/*----------------------------------------------------------------------
 * Special member functions.
 *----------------------------------------------------------------------*/

conversion_function_id:
	OPERATOR conversion_type_id
	;

conversion_type_id:
	type_specifier_seq conversion_declarator_opt
	;

conversion_declarator:
	ptr_operator conversion_declarator_opt
	;

ctor_initializer:
	':' mem_initializer_list
	;

mem_initializer_list:
	mem_initializer
	| mem_initializer ',' mem_initializer_list
	;

mem_initializer:
	mem_initializer_id '(' expression_list_opt ')'
	;

mem_initializer_id:
	COLONCOLON_opt nested_name_specifier_opt class_name
	| identifier
	;

/*----------------------------------------------------------------------
 * Overloading.
 *----------------------------------------------------------------------*/

operator_function_id:
	OPERATOR operator
	;

operator:
	NEW
	| DELETE
	| NEW '[' ']'
	| DELETE '[' ']'
	| '+'
	| '_'
	| '*'
	| '/'
	| '%'
	| '^'
	| '&'
	| '|'
	| '~'
	| '!'
	| '='
	| '<'
	| '>'
	| ADDEQ
	| SUBEQ
	| MULEQ
	| DIVEQ
	| MODEQ
	| XOREQ
	| ANDEQ
	| OREQ
	| SL
	| SR
	| SREQ
	| SLEQ
	| EQ
	| NOTEQ
	| LTEQ
	| GTEQ
	| ANDAND
	| OROR
	| PLUSPLUS
	| MINUSMINUS
	| ','
	| ARROWSTAR
	| ARROW
	| '(' ')'
	| '[' ']'
	;

/*----------------------------------------------------------------------
 * Templates.
 *----------------------------------------------------------------------*/

template_declaration:
	EXPORT_opt TEMPLATE '<' template_parameter_list '>' declaration
	;

template_parameter_list:
	template_parameter
	| template_parameter_list ',' template_parameter
	;

template_parameter:
	type_parameter
	| parameter_declaration
	;

type_parameter:
	CLASS identifier_opt
	| CLASS identifier_opt '=' type_id
	| TYPENAME identifier_opt
	| TYPENAME identifier_opt '=' type_id
	| TEMPLATE '<' template_parameter_list '>' CLASS identifier_opt
	| TEMPLATE '<' template_parameter_list '>' CLASS identifier_opt '=' template_name
	;

template_id:
	template_name '<' template_argument_list '>'
	;

/* template_name: identifier ; */

template_argument_list:
	template_argument
	| template_argument_list ',' template_argument
	;

template_argument:
	assignment_expression
	| type_id
	| template_name
	;

explicit_instantiation:
	TEMPLATE declaration
	;

explicit_specialization:
	TEMPLATE '<' '>' declaration
	;

/*----------------------------------------------------------------------
 * Exception handling.
 *----------------------------------------------------------------------*/

try_block:
	TRY compound_statement handler_seq
	;

function_try_block:
	TRY ctor_initializer_opt function_body handler_seq
	;

handler_seq:
	handler handler_seq_opt
	;

handler:
	CATCH '(' exception_declaration ')' compound_statement
	;

exception_declaration:
	type_specifier_seq declarator
	| type_specifier_seq abstract_declarator
	| type_specifier_seq
	| ELLIPSIS
	;

throw_expression:
	THROW assignment_expression_opt
	;

exception_specification:
	THROW '(' type_id_list_opt ')'
	;

type_id_list:
	type_id
	| type_id_list ',' type_id
	;

/*----------------------------------------------------------------------
 * Epsilon (optional) definitions.
 *----------------------------------------------------------------------*/

declaration_seq_opt:
	/* epsilon */
	| declaration_seq
	;

TEMPLATE_opt:
	/* epsilon */
	| TEMPLATE
	;

nested_name_specifier_opt:
	/* epsilon */
	| nested_name_specifier
	;

expression_list_opt:
	/* epsilon */
	| expression_list
	;

COLONCOLON_opt:
	/* epsilon */
	| COLONCOLON
	;

new_placement_opt:
	/* epsilon */
	| new_placement
	;

new_initializer_opt:
	/* epsilon */
	| new_initializer
	;

new_declarator_opt:
	/* epsilon */
	| new_declarator
	;

expression_opt:
	/* epsilon */
	| expression
	;

statement_seq_opt:
	/* epsilon */
	| statement_seq
	;

condition_opt:
	/* epsilon */
	| condition
	;

decl_specifier_seq_opt:
	/* epsilon */
	| decl_specifier_seq
	;

init_declarator_list_opt:
	/* epsilon */
	| init_declarator_list
	;

identifier_opt:
	/* epsilon */
	| identifier
	;

enumerator_list_opt:
	/* epsilon */
	| enumerator_list
	;

TYPENAME_opt:
	/* epsilon */
	| TYPENAME
	;

initializer_opt:
	/* epsilon */
	| initializer
	;

cv_qualifier_seq_opt:
	/* epsilon */
	| cv_qualifier_seq
	;

exception_specification_opt:
	/* epsilon */
	| exception_specification
	;

constant_expression_opt:
	/* epsilon */
	| constant_expression
	;

abstract_declarator_opt:
	/* epsilon */
	| abstract_declarator
	;

type_specifier_seq_opt:
	/* epsilon */
	| type_specifier_seq
	;

direct_abstract_declarator_opt:
	/* epsilon */
	| direct_abstract_declarator
	;

parameter_declaration_list_opt:
	/* epsilon */
	| parameter_declaration_list
	;

ELLIPSIS_opt:
	/* epsilon */
	| ELLIPSIS
	;

ctor_initializer_opt:
	/* epsilon */
	| ctor_initializer
	;

COMMA_opt:
	/* epsilon */
	| ','
	;

member_specification_opt:
	/* epsilon */
	| member_specification
	;

base_clause_opt:
	/* epsilon */
	| base_clause
	;

member_declarator_list_opt:
	/* epsilon */
	| member_declarator_list
	;

SEMICOLON_opt:
	/* epsilon */
	| ';'
	;

pure_specifier_opt:
	/* epsilon */
	| pure_specifier
	;

constant_initializer_opt:
	/* epsilon */
	| constant_initializer
	;

access_specifier_opt:
	/* epsilon */
	| access_specifier
	;

VIRTUAL_opt:
	/* epsilon */
	| VIRTUAL
	;

conversion_declarator_opt:
	/* epsilon */
	| conversion_declarator
	;

EXPORT_opt:
	/* epsilon */
	| EXPORT
	;

handler_seq_opt:
	/* epsilon */
	| handler_seq
	;

assignment_expression_opt:
	/* epsilon */
	| assignment_expression
	;

type_id_list_opt:
	/* epsilon */
	| type_id_list
	;

%%
   /* post */
static void
yyerror(char *s)
{
	fprintf(stderr, "%d: %s\n", lineno, s);
}

int
main(void)
{
	lineno = 1;
	yyparse();

	return 0;
}
