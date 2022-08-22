use elefren::helpers::cli;
use elefren::prelude::*;
use getopts::Options;

pub struct CliOptions {
    pub help: bool,
    pub register: bool,
    pub follow: bool,
    pub past: bool,
    pub meilisearch: bool,
    pub delete: Option<String>,

    program: String,
    opts: Options,
}

impl CliOptions {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let program = args[0].clone();
        let mut opts = Options::new();

        opts.optflag("h", "help", "print this help menu");
        opts.optflag("f", "follow", "follow live updates.");
        opts.optflag("p", "past", "fetch past updates.");
        opts.optflag("m", "meili", "output to meilisearch");
        opts.optflag("r", "register", "register hunter2 with your instance.");
        opts.optopt("d", "delete", "remove an entry from the index", "TOOT_URL");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => std::panic::panic_any(f.to_string()),
        };

        let help = matches.opt_present("h");
        let register = matches.opt_present("r");

        let meilisearch = matches.opt_present("m");
        let past = matches.opt_present("p");
        let follow = matches.opt_present("f");

        let delete = matches.opt_str("d");

        Self {
            program,
            meilisearch,
            past,
            follow,
            help,
            register,
            delete,
            opts,
        }
    }

    pub fn print_usage(&self) {
        let brief = format!("Usage: {} TEMPLATE_FILE [options]", &self.program);
        print!("{}", &self.opts.usage(&brief));
    }

    pub fn register(&self) {
        let registration = Registration::new(std::env::var("BASE").expect("Read env var BASE"))
            .client_name("hunter2")
            .build()
            .expect("Build registration request");
        let mastodon = cli::authenticate(registration).expect("Attempt to register");

        let message = format!(
            "\
                Save these env vars in .env

                export BASE=\"{}\"\
                export CLIENT_ID=\"{}\"\
                export CLIENT_SECRET=\"{}\"\
                export REDIRECT=\"{}\"\
                export TOKEN=\"{}\"\
        ",
            &mastodon.data.base,
            &mastodon.data.client_id,
            &mastodon.data.client_secret,
            &mastodon.data.redirect,
            mastodon.data.token
        );
        print!("{}", message);
    }
}
