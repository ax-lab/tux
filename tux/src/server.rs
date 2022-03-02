use warp::{path::FullPath, Filter};

pub use warp;

/// Provides a very simple HTTP server that can be used to test HTTP requests.
///
/// The server is bound to the localhost at a random port. The bound port can
/// be retrieved using the `port` method.
pub struct TestServer {
	listen_addr: std::net::SocketAddr,
	inner_state: TestServerState,
}

enum TestServerState {
	Active {
		runtime: tokio::runtime::Runtime,
		server_task: tokio::task::JoinHandle<()>,
		shutdown: tokio::sync::oneshot::Sender<()>,
	},
	Dropped,
}

impl std::ops::Drop for TestServer {
	fn drop(&mut self) {
		// gracefully shutdown the server when the value is dropped by sending
		// a shutdown signal and waiting for the server task to end
		let state = std::mem::replace(&mut self.inner_state, TestServerState::Dropped);
		if let TestServerState::Active {
			runtime,
			server_task,
			shutdown,
		} = state
		{
			shutdown.send(()).expect("sending test server shutdown");
			runtime
				.block_on(server_task)
				.expect("shutting down test server");
		}
	}
}

impl TestServer {
	pub fn port(&self) -> u16 {
		self.listen_addr.port()
	}

	pub fn new_with_root_response(response: &'static str) -> Self {
		let routes = warp::path::end().map(move || response);
		Self::new_with_routes(routes)
	}

	pub fn new_with_ping_route(route: &'static str) -> Self {
		let routes = warp::path(route)
			.and(warp::method().and(warp::path::full()))
			.map(|method, path: FullPath| {
				let mut output = Vec::new();
				output.push(format!("method: {}", method));
				output.push(format!("path: {}", path.as_str()));
				let output = output.join("\n");
				output
			});
		Self::new_with_routes(routes)
	}

	pub fn new_with_routes<F>(routes: F) -> TestServer
	where
		F: warp::Filter + Clone + Send + Sync + 'static,
		F::Extract: warp::Reply,
	{
		let runtime = tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap();

		let (server_task, addr, shutdown) = runtime.block_on(async {
			let (shutdown, wait_shutdown) = tokio::sync::oneshot::channel::<()>();
			let addr = ([127, 0, 0, 1], 0);
			let (addr, server) =
				warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
					wait_shutdown.await.ok();
				});

			let server = runtime.spawn(server);
			(server, addr, shutdown)
		});

		TestServer {
			listen_addr: addr,
			inner_state: TestServerState::Active {
				runtime,
				server_task,
				shutdown,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_should_accept_request() {
		const DATA: &str = "test data";
		let server = TestServer::new_with_root_response(DATA);
		let addr = format!("http://127.0.0.1:{}", server.port());
		let output = get_request_output(addr);
		assert_eq!(output, DATA);
	}

	#[test]
	fn test_server_should_return_404_for_invalid_path() {
		let server = TestServer::new_with_root_response("");
		let addr = format!("http://127.0.0.1:{}/invalid_path", server.port());
		let response_status = reqwest::blocking::get(addr).unwrap().status().as_u16();
		assert_eq!(response_status, 404);
	}

	#[test]
	fn test_server_should_shutdown_on_drop() {
		let server = TestServer::new_with_root_response("");
		let addr = format!("http://127.0.0.1:{}", server.port());
		drop(server);

		let client = reqwest::blocking::ClientBuilder::new();
		let client = client.timeout(std::time::Duration::from_millis(50));
		let client = client.build().unwrap();
		let result = client.get(addr).send();
		assert!(result.is_err());
	}

	#[test]
	fn test_server_with_ping_route_returns_request_info() {
		let server = TestServer::new_with_ping_route("ping");

		let addr = format!("http://127.0.0.1:{}/ping/abc", server.port());
		let addr = &addr;
		let output = get_request_output(addr);
		check_output_contains(&output, "method: GET");
		check_output_contains(&output, "path: /ping/abc");

		let addr = format!("http://127.0.0.1:{}/ping/123", server.port());
		let output = get_post_request_output(addr);
		check_output_contains(&output, "method: POST");
		check_output_contains(&output, "path: /ping/123");

		fn check_output_contains(output: &str, expected: &str) {
			if !output.contains(expected) {
				panic!("output does not contain '{}': {:?}", expected, output);
			}
		}
	}

	fn get_request_output<S: AsRef<str>>(addr: S) -> String {
		let output = reqwest::blocking::get(addr.as_ref())
			.unwrap()
			.bytes()
			.unwrap();
		let output = String::from_utf8_lossy(&output);
		output.to_string()
	}

	fn get_post_request_output<S: AsRef<str>>(addr: S) -> String {
		let addr = reqwest::Url::parse(addr.as_ref()).unwrap();
		let client = reqwest::blocking::ClientBuilder::new().build().unwrap();
		let output = client.post(addr).send().unwrap().bytes().unwrap();
		let output = String::from_utf8_lossy(&output);
		output.to_string()
	}
}
