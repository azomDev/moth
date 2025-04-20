use hotwatch::{EventKind, Hotwatch};
use std::{
	fs,
	io::Read,
	net::TcpListener,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};
use tiny_http::{Response, Server, StatusCode};
use tungstenite::{Message, accept};

use crate::idk::transpile;

type ClientList = Arc<Mutex<Vec<tungstenite::WebSocket<std::net::TcpStream>>>>;

// todo can we make file_path arg a &str?
pub fn serve_hot_reload(file_path: String, ws_port: u16, http_port: u16) {
	// Shared list of WebSocket clients
	let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

	// Spawn WebSocket server thread
	{
		let clients = clients.clone();
		let initial_html_body = transpile(&file_path, false);
		thread::spawn(move || serve_ws(clients, ws_port, &initial_html_body));
	}

	// Watch for file changes
	let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_millis(100))
		.expect("hotwatch failed to initialize!");
	hotwatch
		.watch(file_path.clone(), move |event| {
			if let EventKind::Modify(_) = event.kind {
				let html_data = transpile(&file_path, false);
				notify(&html_data, &clients);
			}
		})
		.unwrap();

	// serve initial HTTP on main thread
	serve_http(http_port, ws_port);
}

pub fn serve_http(http_port: u16, ws_port: u16) {
	let server = Server::http(format!("0.0.0.0:{}", http_port)).unwrap();
	println!("HTTP on http://localhost:{}", http_port);

	for req in server.incoming_requests() {
		let path = req.url();

		if path.starts_with("/img/") {
			// Strip "/img/" from the path to get the file name
			let file_path = format!("img{}", &path[4..]);

			// Try to read the file
			if let Ok(mut file) = fs::File::open(file_path) {
				let mut file_contents = Vec::new();
				if file.read_to_end(&mut file_contents).is_ok() {
					// Serve the PNG image with the correct Content-Type
					let response = Response::from_data(file_contents).with_header(
						tiny_http::Header::from_bytes(b"Content-Type", b"image/png").unwrap(),
					);
					req.respond(response).unwrap();
				} else {
					let response =
						Response::from_string("404 Not Found").with_status_code(StatusCode(404));
					req.respond(response).unwrap();
				}
			} else {
				let response =
					Response::from_string("404 Not Found").with_status_code(StatusCode(404));
				req.respond(response).unwrap();
			}
		} else {
			// Default behavior for non-image requests
			let ct = tiny_http::Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8")
				.unwrap();
			let resp = Response::from_string(initial_html(ws_port)).with_header(ct);
			let _ = req.respond(resp);
		}
	}
}

pub fn serve_ws(clients: ClientList, port: u16, initial_html_body: &str) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
	for stream in listener.incoming() {
		let mut ws = accept(stream.unwrap()).unwrap();
		ws.send(Message::from(initial_html_body)).unwrap();
		clients.lock().unwrap().push(ws);
	}
}

pub fn notify(html_data: &str, clients: &ClientList) {
	// Notify all connected clients to reload
	let mut locked = clients.lock().unwrap();
	for ws in locked.iter_mut() {
		// iter_mut() to get mutable references
		if ws.send(Message::from(html_data)).is_ok() {
			println!("Sent reload to a client");
		}
	}
	println!("Notified {} clients", locked.len());
}

fn initial_html(ws_port: u16) -> String {
	let html = format!(
		r#"
    <!doctype html>
    <html>
        <head>
            <style>
                .small-space {{
                    display: block;
                    height: 0.5em;
                }}
                img {{
                    display: block;
                    max-width: 100%;
                    height: auto;
                }}
                body {{
                    max-width: 1000px;
                    margin: 0 auto;
                }}
                h1,
                h2,
                h3,
                h4,
                h5,
                h6 {{
                    margin-top: 5px;
                    margin-bottom: 5px;
                }}
                p {{
                    margin-top: 5px;
                    margin-bottom: 5px;
                }}
                .custom-br {{
                    height: 15px;
                    display: block;
                    content: "";
                }}
                pre {{
                    background: #f6f8fa;
                    padding: 10px;
                    border-radius: 6px;
                    overflow-x: auto;
                    font-family: monospace;
                    font-size: 14px;
                }}
            </style>
        </head>
        <script>
            const ws = new WebSocket("ws://localhost:{ws_port}");
            ws.onmessage = (evt) => {{
                const start = performance.now();
                console.log("Received message");
                document.body.innerHTML = evt.data;
                const end = performance.now();
                console.log("DOM update took " + (end - start) + "ms");
            }};
        </script>
        <body></body>
    </html>
    "#
	);

	return html;
}
