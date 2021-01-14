//! Fundamental types used by this crate.

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::tokenizer::tag::Tagger;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct OwnedWordId(String, Option<u32>);

impl OwnedWordId {
    pub fn as_ref_id(&self) -> WordId {
        WordId::new(self.0.as_str().into(), self.1)
    }
}

impl AsRef<str> for OwnedWordId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordId<'t>(Cow<'t, str>, Option<u32>);

impl<'t> WordId<'t> {
    pub fn new(text: Cow<'t, str>, id: Option<u32>) -> Self {
        WordId(text, id)
    }

    pub fn to_owned_id(&self) -> OwnedWordId {
        OwnedWordId(self.0.to_string(), self.1)
    }

    pub fn id(&self) -> &Option<u32> {
        &self.1
    }
}

impl<'t> AsRef<str> for WordId<'t> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Lemma and part-of-speech tag associated with a word.
#[derive(Debug, Clone, PartialEq)]
pub struct WordData<'t> {
    pub lemma: WordId<'t>,
    pub pos_id: u16,
}

impl<'t> WordData<'t> {
    pub fn new(lemma: WordId<'t>, pos_id: u16) -> Self {
        WordData { lemma, pos_id }
    }

    pub fn to_owned_word_data(&self) -> OwnedWordData {
        OwnedWordData {
            lemma: self.lemma.to_owned_id(),
            pos_id: self.pos_id,
        }
    }
}

/// An owned version of [WordData] for serialization and use in longer-living structures e. g. rule tests.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct OwnedWordData {
    pub lemma: OwnedWordId,
    pub pos_id: u16,
}

impl OwnedWordData {
    pub fn new(lemma: OwnedWordId, pos_id: u16) -> Self {
        OwnedWordData { lemma, pos_id }
    }
}

/// Contains all the local information about a token i. e.
/// the text itself and the [WordData]s associated with the word.
#[derive(Debug, Clone, PartialEq)]
pub struct Word<'t> {
    pub text: WordId<'t>,
    pub tags: Vec<WordData<'t>>,
}

impl<'t> Word<'t> {
    pub fn new_with_tags(text: WordId<'t>, tags: Vec<WordData<'t>>) -> Self {
        Word { text, tags }
    }

    pub fn to_owned_word(&self) -> OwnedWord {
        OwnedWord {
            text: self.text.to_owned_id(),
            tags: self.tags.iter().map(|x| x.to_owned_word_data()).collect(),
        }
    }
}

/// An owned version of [Word] for serialization and use in longer-living structures e. g. rule tests.
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedWord {
    pub text: OwnedWordId,
    pub tags: Vec<OwnedWordData>,
}

/// A token where varying levels of information are set.
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq)]
pub struct IncompleteToken<'t> {
    pub word: Word<'t>,
    pub byte_span: (usize, usize),
    pub char_span: (usize, usize),
    pub is_sentence_end: bool,
    pub has_space_before: bool,
    pub chunks: Vec<String>,
    pub text: &'t str,
    #[derivative(PartialEq = "ignore", Debug = "ignore")]
    pub tagger: &'t Tagger,
}

/// A finished token with all information set.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Token<'t> {
    pub word: Word<'t>,
    pub char_span: (usize, usize),
    pub byte_span: (usize, usize),
    pub has_space_before: bool,
    pub chunks: Vec<String>,
    pub text: &'t str,
    #[derivative(Debug = "ignore")]
    pub tagger: &'t Tagger,
}

/// An owned version of [Token] for serialization and use in longer-living structures e. g. rule tests.
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedToken {
    pub word: OwnedWord,
    pub char_span: (usize, usize),
    pub byte_span: (usize, usize),
    pub has_space_before: bool,
    pub chunks: Vec<String>,
}

impl<'t> Token<'t> {
    /// Get the special sentence start token.
    pub fn sent_start(text: &'t str, tagger: &'t Tagger) -> Self {
        Token {
            word: Word::new_with_tags(
                tagger.id_word("".into()),
                vec![WordData::new(
                    tagger.id_word("".into()),
                    tagger.tag_to_id("SENT_START"),
                )]
                .into_iter()
                .collect(),
            ),
            char_span: (0, 0),
            byte_span: (0, 0),
            has_space_before: false,
            chunks: Vec::new(),
            text,
            tagger,
        }
    }

    pub fn to_owned_token(&self) -> OwnedToken {
        OwnedToken {
            word: self.word.to_owned_word(),
            char_span: self.char_span,
            byte_span: self.byte_span,
            has_space_before: self.has_space_before,
            chunks: self.chunks.clone(),
        }
    }
}

impl<'t> From<IncompleteToken<'t>> for Token<'t> {
    fn from(data: IncompleteToken<'t>) -> Self {
        let mut word = data.word.clone();

        word.tags.push(WordData::new(
            data.word.text.clone(),
            data.tagger.tag_to_id(""),
        ));

        if word
            .tags
            .iter()
            .all(|x| data.tagger.id_to_tag(x.pos_id).is_empty())
        {
            word.tags.push(WordData::new(
                data.word.text.clone(),
                data.tagger.tag_to_id("UNKNOWN"),
            ));
        }

        if data.is_sentence_end {
            word.tags.push(WordData::new(
                data.word.text,
                data.tagger.tag_to_id("SENT_END"),
            ));
        }

        Token {
            word,
            byte_span: data.byte_span,
            char_span: data.char_span,
            has_space_before: data.has_space_before,
            chunks: data.chunks,
            text: data.text,
            tagger: data.tagger,
        }
    }
}

/// Suggestion for change in a text.
#[derive(Debug, Serialize, Deserialize)]
pub struct Suggestion {
    /// The ID of the rule this suggestion is from.
    pub source: String,
    /// A human-readable message.
    pub message: String,
    /// The start character index (inclusive).
    pub start: usize,
    /// The end character index (exclusive).
    pub end: usize,
    /// The suggested replacement options for the text.
    pub text: Vec<String>,
}
