use clap::Clap;
use nlprule::{
    rules::{Rules, RulesOptions},
    tokenizer::{chunk, tag::Tagger, Tokenizer, TokenizerOptions},
};
use std::{
    collections::HashSet,
    fs::{self, read_to_string},
    io::{BufReader, BufWriter},
};
use std::{fs::File, sync::Arc};

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Benjamin Minixhofer <bminixhofer@gmail.com>"
)]
struct Opts {
    #[clap(long)]
    tag_paths: Vec<String>,
    #[clap(long)]
    tag_remove_paths: Vec<String>,
    #[clap(long)]
    disambiguation_path: String,
    #[clap(long)]
    grammar_path: String,
    #[clap(long)]
    tokenizer_config_path: String,
    #[clap(long)]
    rules_config_path: String,
    #[clap(long)]
    chunker_path: Option<String>,
    #[clap(long)]
    common_words_path: Option<String>,
    #[clap(long)]
    out_tokenizer_path: String,
    #[clap(long)]
    out_rules_path: String,
}

fn main() {
    env_logger::init();
    let opts = Opts::parse();

    let common_words = opts.common_words_path.map_or_else(HashSet::new, |path| {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|x| x.to_string())
            .collect()
    });

    let tokenizer_options: TokenizerOptions =
        serde_json::from_str(&read_to_string(opts.tokenizer_config_path).unwrap()).unwrap();
    let rules_options: RulesOptions =
        serde_json::from_str(&read_to_string(opts.rules_config_path).unwrap()).unwrap();

    let tagger = Tagger::from_dumps(
        &opts.tag_paths,
        &opts.tag_remove_paths,
        &tokenizer_options.extra_tags,
        &common_words,
    )
    .unwrap();
    let tokenizer = Tokenizer::from_xml(
        opts.disambiguation_path,
        Arc::new(tagger),
        if let Some(path) = opts.chunker_path {
            let reader = BufReader::new(File::open(path).unwrap());
            let chunker = chunk::from_json(reader);
            Some(chunker)
        } else {
            None
        },
        tokenizer_options,
    )
    .unwrap();

    let f = BufWriter::new(File::create(&opts.out_tokenizer_path).unwrap());
    bincode::serialize_into(f, &tokenizer).unwrap();

    let rules = Rules::from_xml(opts.grammar_path, tokenizer.tagger(), rules_options);

    let f = BufWriter::new(File::create(&opts.out_rules_path).unwrap());
    bincode::serialize_into(f, &rules).unwrap();
}
