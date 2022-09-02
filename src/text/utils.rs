use bevy			:: prelude :: { * };

use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

extern crate rustc_ast;
extern crate rustc_lexer;
extern crate rustc_span;
extern crate rustc_session;
extern crate rustc_parse;

use rustc_lexer		:: Token;
use rustc_lexer		:: TokenKind;
use rustc_span		:: Span;

use rustc_span		:: edition :: Edition;
use rustc_span		:: BytePos;
use rustc_parse		:: lexer :: nfc_normalize;

pub fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path)	=> path.display().to_string(),
		None		=> String::from(""),
	}
}

pub fn load_text_file(path_str: &str) -> Option<String> {
	let source_file_path = Some(PathBuf::from(path_str));
	let load_name 	= file_path_to_string(&source_file_path);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return None; },
		Ok(file) 	=> file,
	};

	let mut file_content = String::new();
	match file.read_to_string(&mut file_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return None; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

	Some(file_content)
}

pub fn color_from_token_kind(
	token       : &Token,
	token_str   : &str,
	token_start : usize,
	token_end   : usize
) -> Color
{
	let mut color = Color::hex("bbbbbb").unwrap();

	match token.kind {
		TokenKind::Ident => {
			let sym = nfc_normalize(token_str);
			let span = Span::with_root_ctxt(BytePos(token_start as u32), BytePos(token_end as u32));
			let token_kind_ast = rustc_ast::token::TokenKind::Ident(sym, false);
			let token_ast = rustc_ast::token::Token { kind: token_kind_ast, span: span };

			if token_ast.is_used_keyword() {
				color = Color::hex("e06c75").unwrap();
			}

			if token_ast.is_keyword(rustc_span::symbol::kw::Let) {
				color = Color::hex("56b6c2").unwrap();
			}

			if token_ast.is_bool_lit() {
				color = Color::hex("56b6c2").unwrap();
			}

			let chars : Vec<char> = token_str.chars().collect();
			if token.len > 1 && chars[0].is_uppercase() && !chars[1].is_uppercase() {
				color = Color::hex("61afef").unwrap();
			}

			if token.len > 1 && chars[0].is_uppercase() && chars[1].is_uppercase() {
				color = Color::hex("56b6c2").unwrap();
			}

			if token_ast.is_op() {
				println!("what is op? [{}]", token_str);
			}
		},
		TokenKind::OpenBrace | TokenKind::OpenBracket | TokenKind::OpenParen => {
			color = Color::hex("da70d6").unwrap();
		},
		TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen => {
			color = Color::hex("da70d6").unwrap();
		},
		TokenKind::LineComment { doc_style: _ } => {
			color = Color::hex("676f7d").unwrap();
		},
		TokenKind::Literal { kind, suffix_start: _ } => {
			let k = kind;
			match k {
				rustc_lexer::LiteralKind::Int { base: _Base, empty_int: _bool } => {
					color = Color::hex("c678dd").unwrap();
				},
				rustc_lexer::LiteralKind::Float { base: _Base, empty_exponent: _bool } => {
					color = Color::hex("c678dd").unwrap();
				},
				_ => {
					color = Color::hex("e5c07b").unwrap();
				},
			}
		}
		_ => {
			color = Color::hex("e06c75").unwrap();

			let sym = nfc_normalize(token_str);
			let _nt = rustc_ast::token::NonterminalKind::from_symbol(sym, || -> Edition { Edition::Edition2021 });
		},
	}

	color
}