use rinkle::App;

fn main() {
	tracing_subscriber::fmt().init();
	if App::new().run().is_err() {
		std::process::exit(1);
	}
}
