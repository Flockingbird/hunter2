use getopts::Options;

pub struct CliOptions {
    pub help: bool,
    pub register: bool,
    pub follow: bool,
    pub past: bool,
    pub file_name: Option<String>,
    pub meilisearch: bool,

    program: String,
    opts: Options,
}

impl CliOptions {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let program = args[0].clone();
        let mut opts = Options::new();

        opts.optflag("h", "help", "print this help menu");
        opts.optflag("r", "register", "register hunter2 with your instance.");
        opts.optflag("f", "follow", "follow live updates.");
        opts.optflag("p", "past", "fetch past updates.");
        opts.optopt("o", "out", "output to filename as JSONL", "FILE");
        opts.optflag("m", "meili", "output to meilisearch");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => std::panic::panic_any(f.to_string()),
        };

        let help = matches.opt_present("h");
        let register = matches.opt_present("r");

        let meilisearch = matches.opt_present("m");
        let file_name = matches.opt_str("o");
        let past = matches.opt_present("p");
        let follow = matches.opt_present("f");

        Self {
            program,
            file_name,
            meilisearch,
            past,
            follow,
            help,
            opts,
            register,
        }
    }

    pub fn print_usage(&self) {
        let brief = format!("Usage: {} TEMPLATE_FILE [options]", &self.program);
        print!("{}", &self.opts.usage(&brief));
    }
}
