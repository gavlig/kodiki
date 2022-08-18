use std::io::{self, Write, Read};
use std::process::{Command, Stdio};

use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

use std::str::FromStr;
use std::{thread, time};

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

fn run_rust_analyzer() {
	let source_file	= Some(PathBuf::from("playground/easy_spawn.rs"));
	// let source_file	= Some(PathBuf::from("playground/test_letter_spacing.rs"));
	let load_name 	= file_path_to_string(&source_file);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return; },
		Ok(file) 	=> file,
	};

	let mut save_content = String::new();
	match file.read_to_string(&mut save_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

	let mut child = Command::new("assets/lsp/rust-analyzer/rust-analyzer")
	.stdin(Stdio::piped())
	.stdout(Stdio::piped())
	// .stderr(Stdio::piped())
	// .env("RA_LOG", "info")
	.spawn()
	.expect("Failed to spawn child process");
					
	let mut stdin = child.stdin.take().expect("Failed to open stdin");
	let mut stdout = child.stdout.take().expect("Failed to open stdout");
	// let mut stderr = child.stderr.take().expect("Failed to open stderr");
	std::thread::spawn(move || {
		let mut buf = Vec::<u8>::new();
		buf.resize(1024 * 16, 0);

		let mut buf_log = Vec::<u8>::new();
		buf_log.resize(1024 * 1024, 0);

		//
		//

		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "rootPath": "/home/gavlig/workspace/project_gryazevichki/gryazevichki", "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" }, "rust-analyzer.trace.server": "verbose" }}, "id": 1}"#;
		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "rootPath": "/home/gavlig/workspace/project_gryazevichki/gryazevichki", "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;

		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "workspaceFolders": [{ uri: "file:///home/gavlig/workspace/project_gryazevichki/gryazevichki"}], "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;
		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "workspaceFolders": [{ uri: "file:///home/gavlig/workspace/fgl_exercise/bevy_fgl_exercise/"}], "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;

		// let json = format!(r#"{"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": "src/herringbone/spawn.rs", "languageId": "rust", "version": 0, "text": "{}" } }, "id": 3}"#, save_content);
		// let json = r#"{"jsonrpc": "2.0", "method": "textDocument/semanticTokens/full", "params": { "textDocument": { "uri": "file:///home/gavlig/workspace/playground/easy_spawn.rs" } }, "id": 4 }"#;

		#[derive(Serialize, Deserialize, Debug, Clone)]
		struct WorkspaceFolder {
			pub uri: String,
			pub name: String,
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct Synchronization {
			pub dynamicRegistration: bool,
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct SemanticTokensClientRequests {
			pub full : bool,
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct SemanticTokensClientCapabilities {
			pub requests: SemanticTokensClientRequests,
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct TextDocumentClientCapabilities {
			pub synchronization: Synchronization,
			pub semanticTokens: SemanticTokensClientCapabilities,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
        struct TextDocumentItem {
			uri: String,
			languageId: String,
			version: u32,
			text: String,
        }

		#[derive(Serialize, Deserialize, Debug, Clone)]
		struct TextDocumentIdentifier {
			uri: String,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
        struct InitializeParams {
            pub workspaceFolders: Vec<WorkspaceFolder>,
			// pub rootPath: String,
			pub capabilities: TextDocumentClientCapabilities,
        }

		#[derive(Serialize, Deserialize, Debug, Clone)]
        struct DidOpenTextDocumentParams {
			pub textDocument: TextDocumentItem,
        }

		#[derive(Serialize, Deserialize, Debug, Clone)]
        struct SemanticTokensFullParams {
			pub textDocument: TextDocumentIdentifier,
        }

		#[derive(Serialize, Deserialize, Debug, Clone)]
		pub struct RequestInitialize {
			pub id: i32,
			pub method: &'static str,
			pub params: InitializeParams,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
		pub struct RequestSemanticTokensFull {
			pub id: i32,
			pub method: &'static str,
			pub params: SemanticTokensFullParams,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
		pub struct NotificationDidOpenTextDocument {
			pub method: &'static str,
			pub params: DidOpenTextDocumentParams,
		}

		#[derive(Serialize)]
        struct JsonRpcRequestSemanticTokensFull {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: RequestSemanticTokensFull,
        }

		#[derive(Serialize)]
        struct JsonRpcNotificationDidOpenTextDocument {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: NotificationDidOpenTextDocument,
        }

		#[derive(Serialize)]
        struct JsonRpcRequest {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: RequestInitialize,
        }

		///

		let ws = WorkspaceFolder { uri: String::from("file:///home/gavlig/workspace/playground/"), name: String::from("playground") };
		// let ws = WorkspaceFolder { uri: String::from("file:///home/gavlig/workspace/fgl_exercise/bevy_fgl_exercise"), name: String::from("fgl_exercise") };
		// let ws = WorkspaceFolder { uri: String::from("file:///home/gavlig/workspace/project_gryazevichki/gryazevichki/"), name: String::from("gryazevichki") };

		let req = RequestInitialize {
			id: 1,
			method: "initialize",
			params: InitializeParams {
				workspaceFolders: vec![ ws ],
				// rootPath: String::from("file:///home/gavlig/workspace/playground/"),
				capabilities: TextDocumentClientCapabilities {
					synchronization: Synchronization { dynamicRegistration: true },
					semanticTokens: SemanticTokensClientCapabilities { requests: SemanticTokensClientRequests { full: true } },
				}
			}
		};

        let json = serde_json::to_string(&JsonRpcRequest { jsonrpc: "2.0", msg: req }).unwrap();

		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();


		thread::sleep(time::Duration::from_millis(1000));

		println!("KODIKI about to read stdout");
		let read_bytes = stdout.read(&mut buf).unwrap();
		println!("KODIKI read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf.as_slice()));
		buf.fill(0);

		//

		println!("KODIKI sending initialized notification");

		let json = r#"{"jsonrpc": "2.0", "method": "initialized", "params": {}}"#;
		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();

		// 

		thread::sleep(time::Duration::from_millis(1000));

		//
		//

		let not = NotificationDidOpenTextDocument {
			method: "textDocument/didOpen",
			params: DidOpenTextDocumentParams {
				textDocument: TextDocumentItem {
					uri: String::from("file:///home/gavlig/workspace/playground/src/main.rs"),
					// uri: String::from("file:///home/gavlig/workspace/playground/easy_spawn.rs"),
					// uri: String::from("file:///home/gavlig/workspace/fgl_exercise/bevy_fgl_exercise/src/game/systems.rs"),
					//  uri: String::from("file:///home/gavlig/workspace/project_gryazevichki/gryazevichki/src/easy_spawn.rs"),
					
					languageId: String::from("rust"),
					version: 0,
					text: save_content
				}
			}
		};

        let json = serde_json::to_string(&JsonRpcNotificationDidOpenTextDocument { jsonrpc: "2.0", msg: not }).unwrap();

		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		println!("KODIKI sending didOpen notification:\n{}", request);

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();

		

		thread::sleep(time::Duration::from_millis(1000));



		let req = RequestSemanticTokensFull {
			id: 2,
			method: "textDocument/semanticTokens/full",
			params: SemanticTokensFullParams { textDocument: TextDocumentIdentifier { uri: String::from("file:///home/gavlig/workspace/playground/src/main.rs") } }
			// params: SemanticTokensFullParams { textDocument: TextDocumentIdentifier { uri: String::from("file:///home/gavlig/workspace/playground/easy_spawn.rs") } }
			// params: SemanticTokensFullParams { textDocument: TextDocumentIdentifier { uri: String::from("file:///home/gavlig/workspace/fgl_exercise/bevy_fgl_exercise/src/game/systems.rs") } }
			// params: SemanticTokensFullParams { textDocument: TextDocumentIdentifier { uri: String::from("file:///home/gavlig/workspace/project_gryazevichki/gryazevichki/src/easy_spawn.rs") } }
		};

        let json = serde_json::to_string(&JsonRpcRequestSemanticTokensFull { jsonrpc: "2.0", msg: req }).unwrap();

		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		println!("KODIKI sending semantic highlight request");

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();



		thread::sleep(time::Duration::from_millis(20000)); // 1 minutes
		// thread::sleep(time::Duration::from_millis(60000 * 10)); // 10 minutes


		println!("KODIKI ALL DONE");

		loop {
			println!("KODIKI about to read stdout");
			let read_bytes = stdout.read(&mut buf).unwrap();
			println!("KODIKI read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf.as_slice()));
			buf.fill(0);

			thread::sleep(time::Duration::from_millis(3000));
			println!("boolloop");
		}
	});
}