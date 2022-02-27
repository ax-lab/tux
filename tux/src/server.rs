pub struct TestServer {
	addr: std::net::SocketAddr,
	state: ServerState,
}

enum ServerState {
	Active {
		rt: tokio::runtime::Runtime,
		server: tokio::task::JoinHandle<()>,
		shutdown: tokio::sync::oneshot::Sender<()>,
	},
	Inactive,
}

impl TestServer {
	pub fn new(data: &'static str) -> TestServer {
		let rt = tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap();

		let (server, addr, shutdown) = rt.block_on(async {
			use warp::Filter;

			let (shutdown, wait_shutdown) = tokio::sync::oneshot::channel::<()>();
			let root = warp::path::end().map(move || data);
			let addr = ([127, 0, 0, 1], 0);
			let (addr, server) = warp::serve(root).bind_with_graceful_shutdown(addr, async move {
				wait_shutdown.await.ok();
			});

			let server = rt.spawn(server);
			(server, addr, shutdown)
		});

		TestServer {
			addr,
			state: ServerState::Active {
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

impl std::ops::Drop for TestServer {
	fn drop(&mut self) {
		let state = std::mem::replace(&mut self.state, ServerState::Inactive);
		if let ServerState::Active {
			rt,
			server,
			shutdown,
		} = state
		{
			shutdown.send(()).expect("sending test server shutdown");
			rt.block_on(server).expect("shutting down server");
		}
	}
}
