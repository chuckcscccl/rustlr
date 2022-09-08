// F# parser writer (assuming no changes to grammar processor
// unit type must be changed to obj

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::io::{self,Read,Write,BufReader,BufRead};
use std::collections::{HashSet,HashMap};
//use std::hash::{Hash,Hasher};
//use std::any::Any;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
//use std::mem;
use crate::{Statemachine,checkboxlabel};
use crate::Stateaction::*;

const UNITTYPE:&'static str = "unit";  // "unit"

/////////////////////LRSD VERSION FOR F#///////////////////////////////////
   ///// semantic action fn is _rrsemaction_rule_{rule index}
////////////////////////////////////////////////
impl Statemachine
{

  fn re_transform(&mut self)
  {
     let Gmr = &mut self.Gmr;
     let mut ntcx = Gmr.ntcxmax + 1;
// must set passthru type for newseqnt's first, otherwise other actions
// won't konw their type
     for (nti,ntrules) in Gmr.Rulesfor.iter() {
       if Gmr.Symbols[*nti].sym.starts_with("NEWSEQNT_") {   // (E ;)
         let mut newtype = String::from("unit");
         // must determine passthru independently
         let mut passthru:i32 = -1;  // allowed only for single non-terminal,
         let mut rsymi = 0;
         // or terminal that is not same as absyntype (for now)
         assert!(Gmr.Rulesfor.get(nti).unwrap().len()==1);
         for nri in Gmr.Rulesfor.get(nti).unwrap().iter() { // 1 rule
           for i in 0.. Gmr.Rules[*nri].rhs.len() {
             let rsym = &Gmr.Rules[*nri].rhs[i];
             if passthru==-1 && ((!rsym.terminal && rsym.rusttype!="()") || (&rsym.rusttype!="()" && (&rsym.rusttype != &Gmr.Absyntype))) { passthru = i as i32; rsymi = Gmr.Rules[*nri].rhs[i].index; }
             else if passthru>=0 && ((!rsym.terminal && rsym.rusttype!="()") || (&rsym.rusttype!="()" && &rsym.rusttype!=&Gmr.Absyntype) || rsym.precedence!=0) {passthru=-2;}
           }//for each rhs symbol for single rule for NESEQNT
           if passthru>=0 {
             Gmr.Symbols[*nti].rusttype = Gmr.Symbols[rsymi].rusttype.clone();
             Gmr.Rules[*nri].action = format!(" _item{}_ }}",passthru);
           }
         }// for single rule nri for NEWSEQNT
         if passthru<0 {eprintln!("ERROR: SEQUENCES ENCLOSED IN (..) MUST HAVE EXACTLY ONE NON-DEFAULT TYPE SYMBOL");}
//         else {
           //println!("passthru found at {}, type {}",passthru,&Gmr.Symbols[*nti].rusttype);
//         }
       } // is NEWSEQNT
     } // first pass to look for NEWSEQNT's

     let mut newretypes = HashMap::new();
     for (nti,ntrules) in Gmr.Rulesfor.iter() {
       if Gmr.Symbols[*nti].sym.starts_with("NEWRENT_") || Gmr.Symbols[*nti].sym.starts_with("NEWSEPNT") { //is for *, + or ?
         for nri in ntrules.iter() {
           if Gmr.Symbols[*nti].rusttype.starts_with("Option<") {
             if Gmr.Rules[*nri].rhs.len()==0 {
               Gmr.Rules[*nri].action = " None }".to_owned();
             } // change action for Option NT
             else if Gmr.Rules[*nri].rhs.len()==1 {
               Gmr.Rules[*nri].action = " Some(_item0_) }".to_owned();
               let targetindex = Gmr.Rules[*nri].rhs[0].index;
               let targettype = &Gmr.Symbols[targetindex].rusttype;
               Gmr.Symbols[*nti].rusttype = format!("{} option",targettype);
               if !Gmr.enumhash.contains_key(&Gmr.Symbols[*nti].rusttype) {
                 Gmr.enumhash.insert(Gmr.Symbols[*nti].rusttype.clone(),ntcx);
                 ntcx+=1;
               } // register type
             } //rhs.len is 1
           } //is of option type
           else if Gmr.Symbols[*nti].rusttype.starts_with("Vec<") && Gmr.Rules[*nri].rhs.len()>=2 { // sets type first
             let lasti = Gmr.Rules[*nri].rhs.len()-1;
             let targetindex = Gmr.Rules[*nri].rhs[lasti].index;
             let targettype = Gmr.Symbols[targetindex].rusttype.clone();
             Gmr.Symbols[*nti].rusttype = format!("Vec<{}>",&targettype);
             if !Gmr.enumhash.contains_key(&Gmr.Symbols[*nti].rusttype) {
               Gmr.enumhash.insert(Gmr.Symbols[*nti].rusttype.clone(),ntcx);
               ntcx+=1;
             } // register type             
             Gmr.Rules[*nri].action = format!(" (_item0_.Add(_item{}_); _item0_) }}",lasti);
             newretypes.insert(*nti,targettype);
           } // if for  + or *
         }// for each rule of this NEWRENT
       }// is NEWRENT
     }// for each (nt,ntrules) in Rulesfor
     // third pass sets actions for NEWRENT's
     for (nti,targettype) in newretypes.iter() {
       for nri in Gmr.Rulesfor.get(nti).unwrap() {
         if Gmr.Rules[*nri].rhs.len()==0 {
           Gmr.Rules[*nri].action = format!(" Vec<{}>() }}",targettype);
         } // rhs len 0
         else if Gmr.Rules[*nri].rhs.len()==1 && !Gmr.Symbols[*nti].sym.starts_with("NEWSEPNT2_") {
           Gmr.Rules[*nri].action = format!(" let _yyv = Vec<{}>() in (_yyv.Add(_item0_); _yyv) }}",targettype);
         } // else action is correct: _item0_
       } // for each rule for nti
     }//third pass
     Gmr.ntcxmax = ntcx;
  }// transforms grammar NEWRENT's to have F# types and actions


///////////////// main writeparser function

  pub fn writefsparser(&mut self, filename:&str)->Result<(),std::io::Error>
  {

    self.re_transform(); // change type and actions for NEWRENTs

    let mut absyn = self.Gmr.Absyntype.as_str();
    let mut extype = self.Gmr.Externtype.as_str();
    if absyn=="()" {absyn=UNITTYPE;}
    if extype=="()" {extype=UNITTYPE;}
    let rlen = self.Gmr.Rules.len();
    
    // generate action fn's from strings stored in gen-time grammar
    // make this a pure function on types defined.
    let mut actions:Vec<String> = Vec::with_capacity(rlen);
    
    for ri in 0..rlen
    {
      let lhs = &self.Gmr.Rules[ri].lhs.sym;
      let lhsi = self.Gmr.Rules[ri].lhs.index;
      // rusttype must now represent a F# type
      let mut rettype = self.Gmr.Symbols[lhsi].rusttype.as_str(); // return type
      if rettype=="()" {rettype=UNITTYPE;}

// first arg to semaction is parser itself. - this is a must.
      let mut fndef = format!("let _rrsemaction_{}_(parser:RTParser<FLTypeDUnion,{}>",ri,extype);
      // now for other arguments
      // inside actions, can still bind labels to patterns
      for k in 0..self.Gmr.Rules[ri].rhs.len() {
        let symk= &self.Gmr.Rules[ri].rhs[k]; 
        let mut symktype = self.Gmr.Symbols[symk.index].rusttype.as_str();
        if symktype=="()" {symktype=UNITTYPE;}
        let(labelkind,label) = decode_label(&symk.label,k);
        if labelkind!=0 {panic!("ONLY SIMPLE LABELS ARE SUPPORTED IN F# GRAMMARS");}
        let mut fargk = format!(", {}:{}",&label,symktype);
        //match labelkind only implemented for type 0
        fndef.push_str(&fargk);
      }// for each symbol on rhs
      fndef.push_str(") =\n ");

      // ALL SEMANTIC ACTIONS WILL RETURN OPTION TYPES? NO.
      let defaultaction = format!("  Unchecked.defaultof<{}>",rettype);
      let mut semaction = self.Gmr.Rules[ri].action.as_str(); // ends w/ rbr
      if let Some(rbrpos) = semaction.rfind('}') { // REMOVING } from action
        semaction = &self.Gmr.Rules[ri].action[..rbrpos];
      }
      if semaction.len()<1 {semaction = &defaultaction;}
      fndef.push_str(semaction);
      //fndef.push('\n');
      actions.push(fndef);
    } //for ri

    ////// write to file, create Ruleaction closures for each rule

    let mut firstchar = self.Gmr.name.chars().next().unwrap();
    firstchar.make_ascii_uppercase();
    
    let mut fd = File::create(filename)?;
    write!(fd,"//F# Parser generated by Rustlr for grammar {}",&self.Gmr.name)?;
    write!(fd,"\n    
module {}{}
open System;
open System.Collections.Generic;
open Fussless;
open Fussless.RuntimeParser;\n",firstchar,&self.Gmr.name[1..])?;

    write!(fd,"{}\n",&self.Gmr.Extras)?; // verbatim code better be in F#

    // write static array of symbols
    write!(fd,"let private SYMBOLS = [|")?;
    for i in 0..self.Gmr.Symbols.len()-1
    {
      write!(fd,"\"{}\";",&self.Gmr.Symbols[i].sym)?;
    }
    write!(fd,"\"{}\"|];\n\n",&self.Gmr.Symbols[self.Gmr.Symbols.len()-1].sym)?;
    // position of symbols must be inline with self.Gmr.Symhash

    // record table entries in a static array
    let mut totalsize = 0;
    for i in 0..self.FSM.len() { totalsize+=self.FSM[i].len(); }
if self.Gmr.tracelev>1 {println!("{} total state table entries",totalsize);}
    write!(fd,"let private TABLE:uint64 [] = [|")?;
    // generate table to represent FSM
    let mut encode:u64 = 0;
    for i in 0..self.FSM.len() // for each state index i
    {
      let row = &self.FSM[i];        
      for key in row.keys()
      { // see function decode for opposite translation
        let k = *key; //*self.Gmr.Symhash.get(key).unwrap(); // index of symbol
        encode = ((i as u64) << 48) + ((k as u64) << 32);
        match row.get(key) {
          Some(Shift(statei)) => { encode += (*statei as u64) << 16; },
          Some(Gotonext(statei)) => { encode += ((*statei as u64) << 16)+1; },
          Some(Reduce(rulei)) => { encode += ((*rulei as u64) << 16)+2; },
          Some(Accept) => {encode += 3; },
          _ => {encode += 4; },  // 4 indicates Error
        }//match
        write!(fd,"{}UL;",encode)?;
      } //for symbol index k
    }//for each state index i
    write!(fd," |];\n\n")?;

    ////// WRITE ENUM 
    self.gen_fsunion(&mut fd)?;

    // write action functions fn _semaction_rule_{} ..
    for deffn in &actions { write!(fd,"{}\n\n",deffn)?; }

    write!(fd,"let make_parser() : RTParser<FLTypeDUnion,{}> =\n",extype)?; 
    write!(fd,"  let parser1 = skeleton_parser(Unchecked.defaultof<{}>,{},{})\n",extype,self.Gmr.Rules.len(),self.FSM.len())?;
    // generate rules and Ruleaction delegates to call action fns, cast
    write!(fd,"  let mutable rule = skeleton_production(\"\")\n")?; //init dummy
    for ri in 0..self.Gmr.Rules.len() 
    {
      write!(fd,"  rule <- skeleton_production(\"{}\")\n",&self.Gmr.Rules[ri].lhs.sym)?;
      write!(fd,"  rule.action <- fun parser ->\n    (")?;

    // write code to pop stack, decode labels into args. /////////
      let mut k = self.Gmr.Rules[ri].rhs.len(); //k=len of rhs of rule ri
      //form if-let labels and patterns as we go...
      let mut actualargs = Vec::new();
      while k>0 // k is length of right-hand side, use k-1
      {
        let gsym = &self.Gmr.Rules[ri].rhs[k-1]; // rhs syms right to left
        let (lbtype,poppedlab) = decode_label(&gsym.label,k-1);
        let mut symtype=self.Gmr.Symbols[gsym.index].rusttype.as_str();
        if symtype=="()" {symtype=UNITTYPE;}
        let emsg = format!("FATAL ERROR: '{}' IS NOT A TYPE IN THIS GRAMMAR. DID YOU INTEND TO USE THE -auto OPTION TO GENERATE TYPES?",&symtype);
        let eindex = self.Gmr.enumhash.get(&self.Gmr.Symbols[gsym.index].rusttype).expect(&emsg);
        actualargs.push(format!("{}",&poppedlab));
        let stat = format!("let {0} = (match parser.Pop().svalue with | FLTypeDUnion.Enumvariant_{1}(_rr_{1}) ->  _rr_{1} | _ -> Unchecked.defaultof<{2}>) in ",&poppedlab,&eindex,symtype); // only for simple labels
        write!(fd,"{}",&stat)?;
        k-=1;
      } // while k>0
      // form args
      let mut aargs = String::new(); // call to semaction function
      k = actualargs.len();
      while k>0
      {
        aargs.push(',');
        aargs.push_str(&actualargs[k-1]);
        k-=1;
      }
      /// formed actual arguments
    // write code to call action function, then convert to FLTypeDUnion
      let lhsi = self.Gmr.Symhash.get(&self.Gmr.Rules[ri].lhs.sym).expect("GRAMMAR REPRESENTATION CORRUPTED");
      let fnname = format!("_rrsemaction_{}_",ri);
      let mut typei = self.Gmr.Symbols[*lhsi].rusttype.as_str();
      if typei=="()" {typei=UNITTYPE;}
      let enumindex = self.Gmr.enumhash.get(&self.Gmr.Symbols[*lhsi].rusttype).expect(&format!("FATAL ERROR: TYPE {} NOT USED IN GRAMMAR",typei));
      write!(fd," FLTypeDUnion.Enumvariant_{}({}(parser{})));\n",enumindex,&fnname,aargs)?;
      write!(fd,"  parser1.Rules.[{}] <- rule;\n",ri)?;
    }// write each rule action
    
    
    //write!(fd," parser1.Errsym = \"{}\";\n",&self.Gmr.Errsym)?;
    // resynch vector
    for s in &self.Gmr.Resynch {write!(fd,"  ignore (parser1.resynch.Add(\"{}\"));\n",s)?;}

    // generate code to load RSM from TABLE
    write!(fd,"\n  for i in 0..{} do\n",totalsize-1)?; //F# ranges inclusive
    write!(fd,"    let (sti,symi,action) = decode_action(TABLE.[i])\n")?;
    write!(fd,"    parser1.RSM.[sti].Add(SYMBOLS.[symi],action)\n")?;
    write!(fd,"  for s in SYMBOLS do ignore (parser1.Symset.Add(s));\n")?;
    
//    write!(fd,"  load_extras(parser1);\n")?;
    write!(fd,"  parser1;;\n")?;


/////////////////////////////


//////////////////// write convert_token function
     write!(fd,"\nlet convert_token (lt:RawToken) =\n  if lt=null then None\n  else\n    let (uval,utype) = \n      match lt.token_name with\n")?;
     let abindex = self.Gmr.enumhash.get(&self.Gmr.Absyntype).expect("F absyn - Sharp!");
     let unitindex = self.Gmr.enumhash.get("()").expect("F absyn - Sharp!");
     for (terminalname,tokentype,valfun) in &self.Gmr.Lexvals {
       let symi = *self.Gmr.Symhash.get(terminalname).unwrap();
       let sym = &self.Gmr.Symbols[symi];
       let eindex = self.Gmr.enumhash.get(&sym.rusttype).expect("F- Sharp!");
       if /* stype!=UNITTYPE && */ &sym.sym!="EOF" {
         write!(fd,"        | \"{}\" -> (FLTypeDUnion.Enumvariant_{}({}(lt.token_text)),\"{}\")\n",tokentype.trim(),eindex,valfun.trim(),terminalname)?;
       }  // has been declared like valueterminal~ num~ int~ n int(n)
     } //for (name,form,val) entry in Lexvals
     // for lexterminals:
     // assuming that for these the lt.token_name and lt_token_text are same
     for (textform,termname) in self.Gmr.Lexnames.iter() {
        let tsymi = *self.Gmr.Symhash.get(termname).unwrap();
        let tsym = &self.Gmr.Symbols[tsymi];
        let eindex = self.Gmr.enumhash.get(&tsym.rusttype).expect("F-Sharp3!");
        let mut ttype = tsym.rusttype.as_str();
        if ttype=="()" {ttype=UNITTYPE;}
        write!(fd,"        | \"{}\" -> (FLTypeDUnion.Enumvariant_{}(Unchecked.defaultof<{}>),\"{}\")\n",textform,eindex,ttype,termname)?;
     }//for Lexnames
     ///// now for other terminals, token type expected to be Symbol? NO
     //for now, expect type and text to be the same
     for i in 1..self.Gmr.Symbols.len() {  // skip wildcard
       let sym = &self.Gmr.Symbols[i];
       if !sym.terminal || self.Gmr.Haslexval.contains(&sym.sym) {continue;}
       let eindex = self.Gmr.enumhash.get(&sym.rusttype).expect("F- Sharp 2!");
       let mut stype = sym.rusttype.as_str();
       if stype=="()" {stype=UNITTYPE;}
       write!(fd,"        | \"{}\" -> (FLTypeDUnion.Enumvariant_{}(Unchecked.defaultof<{}>),\"{}\")\n",&sym.sym,eindex,stype,&sym.sym)?;
     }//terminals not in lexvals
     write!(fd,"        | x -> (FLTypeDUnion.Enumvariant_{}(Unchecked.defaultof<{}>),\"Error:\"+x)\n",abindex,absyn)?;
     write!(fd,"    Some({{TerminalToken.sym=utype; svalue=uval; line=lt.line; column=lt.column;}});;\n")?;
     


      ////// WRITE parse_with
      let abindex = *self.Gmr.enumhash.get(&self.Gmr.Absyntype).unwrap();
      write!(fd,"\nlet parse_with(parser:RTParser<FLTypeDUnion,{1}>, lexer:AbstractLexer<{1}>) : {0} option  =\n",absyn,extype)?;
      write!(fd,"  lexer.set_shared(parser.exstate)\n")?;
      write!(fd,"  parser.NextToken <- fun () -> convert_token(lexer.next_lt())\n")?;
      write!(fd,"  match parser.parse_core() with\n")?;
      write!(fd,"    | Some(FLTypeDUnion.Enumvariant_{}(_yyxres_)) -> Some(_yyxres_)\n",abindex)?;
      write!(fd,"    | _ -> None;;\n\n")?;


      /*    
      // training version
      write!(fd,"\npub fn parse_train_with{}(parser:&mut ZCParser<FLTypeDUnion{},{}>, lexer:&mut {}, parserpath:&str) -> Result<{},{}>\n{{\n",lexerlt,&ltopt,extype,&lexername,absyn,absyn)?;
      write!(fd,"  lexer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let FLTypeDUnion::Enumvariant_{}(_xres_) = parser.parse_train(lexer,parserpath) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_train_with public function\n",absyn)?;
      */

    ////// Augment!
//    write!(fd,"\n  let load_extras(parser:RTParser<FLTypeDUnion,{}>) =\n    ();\n",extype)?;
//    write!(fd,"  //end of load_extras: don't change this line as it affects augmentation\n")?;

     //////// generate lexer in a different file
     if self.Gmr.genlex {
       // extract path from filename
       let mut lexpath = "";
       if let Some(rpos)=filename.rfind('/') {
          lexpath = &filename[..rpos+1];
       }else if let Some(rpos)=filename.rfind('\\') {
          lexpath = &filename[..rpos+1];
       }
       let mut lexfd = File::create(&format!("{}{}.lex",lexpath,&self.Gmr.name))?;
       if let Err(e) = self.gencslex(&mut lexfd) {eprintln!("ERROR GENERATING .lex, {:?}",&e);}
       else {println!("Created {}{}.lex",lexpath,&self.Gmr.name);}
     }

    Ok(())
  }//writefsparser


// generates the union type unifying absyntype. - F# version
pub fn gen_fsunion(&self,fd:&mut File) -> Result<(),std::io::Error>
{
    let symlen = self.Gmr.Symbols.len();
    write!(fd,"\n//Enum for return values \ntype FLTypeDUnion = ")?;

    for (typesym1,eindex) in self.Gmr.enumhash.iter()
    {
       let mut typesym = typesym1.as_str();
       if typesym=="()" {typesym=UNITTYPE;}
       else if typesym=="(usize,usize)" {typesym="int*int";}
       write!(fd,"| Enumvariant_{} of {} ",eindex,typesym)?;
    }
    write!(fd,";;\n\n")?;
    Ok(())
}// generate enum from rusttype defs FLTypeDUnion::Enumvariant_0 is absyntype


/////////////// auto genlex option

// generated .lex file to be processed by CSLex. follows template in
// test1.lex
pub fn gencslex(&self,fd:&mut File) -> Result<(),std::io::Error>
{
   write!(fd,"//CsLex file generated from grammar {}\n",&self.Gmr.name)?;
   write!(fd,"#pragma warning disable 0414
using System;
using System.Text;\n\n")?;
   write!(fd,"public class {}lexer<ET> : AbstractLexer<ET>  {{\n",&self.Gmr.name)?;
   write!(fd,"  Yylex lexer;\n  ET shared_state;\n")?;
   write!(fd,"  public {}lexer(string n) {{ lexer = new Yylex(new System.IO.StringReader(n)); }}\n",&self.Gmr.name)?;
   write!(fd,"  public {}lexer(System.IO.FileStream f) {{ lexer=new Yylex(f); }}\n",&self.Gmr.name)?;
   write!(fd,"  public RawToken next_lt() => lexer.yylex();\n  public void set_shared(ET shared) {{shared_state=shared;}}\n}}//lexer class\n\n")?;
   write!(fd,"{}\n",r#"%%
%namespace Fussless
%type RawToken
%eofval{
  return new RawToken("EOF","EOF",yyline,yychar);
%eofval}  
%{
private static int comment_count = 0;
private static int line_char = 0;
%}
%line
%char
%state COMMENT

ALPHA=[A-Za-z]
DIGIT=[0-9]
DIGITS=[0-9]+
FLOATS = [0-9]*\.[0-9]+([eE]([+-]?){DIGITS})?
HEXDIGITS=(0x)[0-9A-Fa-f]*
NEWLINE=((\r\n)|\n)
NONNEWLINE_WHITE_SPACE_CHAR=[\ \t\b\012]
WHITE_SPACE_CHAR=[{NEWLINE}\ \t\b\012]
STRING_TEXT=(\\\"|[^{NEWLINE}\"]|{WHITE_SPACE_CHAR}+)*
COMMENT_TEXT=([^*/\r\n]|[^*\r\n]"/"[^*\r\n]|[^/\r\n]"*"[^/\r\n]|"*"[^/\r\n]|"/"[^*\r\n])*
ALPHANUM=[A-Za-z_][A-Za-z0-9_]*
"#)?;

  write!(fd,"{}",r#"%% 
<YYINITIAL> {NEWLINE}+ { line_char = yychar+yytext().Length; return null; }
<YYINITIAL> {NONNEWLINE_WHITE_SPACE_CHAR}+ { return null; }
"#)?;

  //////////// now for all terminals
  // write Lexnames forms first
  for form in self.Gmr.Lexnames.keys() {
    write!(fd,"<YYINITIAL> \"{0}\" {{ return new RawToken(\"{0}\",yytext(),yyline,yychar-line_char,yychar); }}\n",form)?;
  }
  for i in 1..self.Gmr.Symbols.len() {
     if i==self.Gmr.eoftermi || !self.Gmr.Symbols[i].terminal || self.Gmr.Haslexval.contains(&self.Gmr.Symbols[i].sym) {continue;}
     write!(fd,"<YYINITIAL> \"{0}\" {{ return new RawToken(\"{0}\",yytext(),yyline,yychar-line_char,yychar); }}\n",&self.Gmr.Symbols[i].sym)?;
  }// for all terminals on in lexnames list

  let mut linecomment = "//"; // if there is one
/// write custom tokens: lexattribute custom ULong regex
  for attribute in &self.Gmr.Lexextras {
    let asplit:Vec<_> = attribute.split_whitespace().collect();
    if asplit.len()>=3 && asplit[0]=="custom" {
       let tokenname = asplit[1];
       let mut re = String::new();
       for i in 2..asplit.len() {
         re.push_str(asplit[i]); re.push(' ');
       }   // need trim
       write!(fd,"<YYINITIAL> {} {{ return new RawToken(\"{}\",yytext(),yyline,yychar-line_char,yychar); }}\n",re.trim(),tokenname)?;
    } // long enough
    else if asplit.len()==2 && asplit[0]=="line_comment" {
      linecomment = asplit[1].trim();
    }
  }//for possible custom token
  ///////////////// customs

  write!(fd,"\n<YYINITIAL> \"{}\".*\\n {{ line_char=yychar+yytext().Length; return null; }}\n",linecomment)?;
// <YYINITIAL> "//".*\n { line_char=yychar+yytext().Length; return null; }

  write!(fd,"{}\n",r#"<YYINITIAL,COMMENT> [(\r\n?|\n)] { line_char=yychar+yytext().Length; return null; }

<YYINITIAL> "/*" { yybegin(COMMENT); comment_count = comment_count + 1; return null;
}
<COMMENT> "/*" { comment_count = comment_count + 1; return null; }
<COMMENT> "*/" { 
	comment_count = comment_count - 1;
	if (comment_count == 0) {
            yybegin(YYINITIAL);
        }
        return null;
}

<COMMENT> {COMMENT_TEXT} { return null; }

<YYINITIAL> \"{STRING_TEXT}\" {
        return new RawToken("StrLit",yytext(),yyline,yychar-line_char,yychar);
}
<YYINITIAL> \"{STRING_TEXT} {
	String str =  yytext().Substring(1,yytext().Length);
	Utility.error(Utility.E_UNCLOSEDSTR);
        return new RawToken("Unclosed String",str,yyline,yychar-line_char,yychar);
}
"#)?;

  //// important categories
  write!(fd,"{}",r#"<YYINITIAL> {DIGIT}+ { 
  return new RawToken("Num",yytext(),yyline,yychar-line_char,yychar);
}
<YYINITIAL> {HEXDIGITS} { 
return new RawToken("Hexnum",yytext(),yyline,yychar-line_char,yychar);  
}
<YYINITIAL> {FLOATS} { 
  return new RawToken("Float",yytext(),yyline,yychar-line_char,yychar);
}	
<YYINITIAL> ({ALPHA}|_)({ALPHA}|{DIGIT}|_)* {
        return new RawToken("Alphanum",yytext(),yyline,yychar-line_char,yychar);
}	
<YYINITIAL,COMMENT> . {
	StringBuilder sb = new StringBuilder("Illegal character: <");
	String s = yytext();
	for (int i = 0; i < s.Length; i++)
	  if (s[i] >= 32)
	    sb.Append(s[i]);
	  else
	    {
	    sb.Append("^");
	    sb.Append(Convert.ToChar(s[i]+'A'-1));
	    }
        sb.Append(">");
	Console.WriteLine(sb.ToString());	
	Utility.error(Utility.E_UNMATCHED);
        return null;
}
"#)?;

   Ok(())
}//gencslex

}//impl statemachine

// decode a grammar label, first return value is type of the label
// 0=direct
// 1=boxed
// 2= &mut   like in e@..@
// 3= &mut box  like in [e]@..@
// 4= no distinct label, @..@ without name
// k = position of argument of rhs 0 = first
pub fn decode_label(label:&str,k:usize) -> (u8,String)
{
  let mut plab = format!("_item{}_",k);
  if label.len()==0 {return (0,plab);}
  let mut boxedlabel = false;  // see if named label is of form [x]
  let findat = label.find('@');
  let mut ltype = 0;
  match &findat {
     None if label.len()>0 /*&& !gsym.label.contains('(')*/ => {
            let truelabel = checkboxlabel(label);
            boxedlabel = truelabel != label; 
            plab = String::from(truelabel);
            if boxedlabel {ltype=1;} /* else {ltype=0;} */
          },
    Some(ati) if *ati==0 => { ltype=4; },
    Some(ati) if *ati>0 => {
            let rawlabel = label[0..*ati].trim();
            let truelabel = checkboxlabel(rawlabel);
            boxedlabel = truelabel != rawlabel;
            if boxedlabel {ltype=3;} else {ltype=2;}
            plab = String::from(truelabel);
          },
    _ => {},
  }//match
  if ltype>1
    {eprintln!("\nWARNING: @..@ PATTERNS MUST BE IRREFUTABLE WITH THE -lrsd OPTION\n");}
  //if plab.starts_with("NEW") {plab=format!("_item{}_",k);}
  (ltype,plab)
}//decode_label


