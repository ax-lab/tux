use warp::Filter;

pub use warp;

/// Provides a very simple HTTP server that can be used to test HTTP requests.
///
/// The server is bound to the localhost at a random port. The bound port can
/// be retrieved using the `port` method.
pub struct TestServer {
	addr: std::net::SocketAddr,
	state: TestServerState,
}

enum TestServerState {
	Active {
		rt: tokio::runtime::Runtime,
		server: tokio::task::JoinHandle<()>,
		shutdown: tokio::sync::oneshot::Sender<()>,
	},
	Inactive,
}

impl std::ops::Drop for TestServer {
	fn drop(&mut self) {
		let state = std::mem::replace(&mut self.state, TestServerState::Inactive);
		if let TestServerState::Active {
			rt,
			server,
			shutdown,
		} = state
		{
			shutdown.send(()).expect("sending test server shutdown");
			rt.block_on(server).expect("shutting down test server");
		}
	}
}

impl TestServer {
	pub fn new_with_root_response(response: &'static str) -> TestServer {
		let routes = warp::path::end().map(move || response);
		Self::new_with_routes(routes)
	}

	pub fn new_with_routes<F>(routes: F) -> TestServer
	where
		F: warp::Filter + Clone + Send + Sync + 'static,
		F::Extract: warp::Reply,
	{
		let rt = tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap();

		let (server, addr, shutdown) = rt.block_on(async {
			let (shutdown, wait_shutdown) = tokio::sync::oneshot::channel::<()>();
			let addr = ([127, 0, 0, 1], 0);
			let (addr, server) =
				warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
					wait_shutdown.await.ok();
				});

			let server = rt.spawn(server);
			(server, addr, shutdown)
		});

		TestServer {
			addr,
			state: TestServerState::Active {
				rt,
				server,
				shutdown,
			},
		}
	}

	pub fn port(&self) -> u16 {
		self.addr.port()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_should_accept_simple_request() {
		const DATA: &str = "test data";
		let server = TestServer::new_with_root_response(DATA);
		let addr = format!("http://127.0.0.1:{}", server.port());
		let output = reqwest::blocking::get(addr).unwrap().bytes().unwrap();
		let output = String::from_utf8_lossy(&output);
		assert_eq!(output, DATA);
	}

	#[test]
	fn test_server_should_return_404_for_invalid_path() {
		let server = TestServer::new_with_root_response("");
		let addr = format!("http://127.0.0.1:{}/invalid_path", server.port());
		let response_status = reqwest::blocking::get(addr).unwrap().status().as_u16();
		assert_eq!(response_status, 404);
	}
}
