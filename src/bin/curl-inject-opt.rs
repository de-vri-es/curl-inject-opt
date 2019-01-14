use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Set curl options for a subcommand.", author = "")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Args {
	#[structopt(long = "client-cert")]
	client_cert: Option<String>,

	#[structopt(long = "client-key")]
	client_key: Option<String>,

	#[structopt(name = "COMMAND")]
	command: Vec<String>,
}

fn main() {
	let args = Args::from_args();
	println!("{:?}", args);
}
