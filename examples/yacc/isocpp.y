%token identifier
%token COLONCOLON "::"
%start id_expression
%%
/* 1.4  Expressions */

id_expression:
  unqualified_id |
  qualified_id ;

unqualified_id:
  identifier |
  template_id ;

qualified_id:
  nested_name_specifier unqualified_id ;

nested_name_specifier:
   class_name "::" nested_name_specifier |
   class_name "::" ;

/* 1.8  Classes */

class_name:
  identifier |
  template_id ;

/* 1.12  Templates */

template_id:
  identifier '<' template_argument '>' ;

template_argument:
  id_expression ;
