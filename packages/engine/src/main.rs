use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};

use serde_json::Result;

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();
        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()));
        }

        if self.content[0].is_alphabetic() {
            return Some(self.chop_while(|x| x.is_alphanumeric()));
        }

        return Some(self.chop(1));
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn index_document(_doc_content: &str) -> HashMap<String, usize> {
    todo!("not Implemented")
}

fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let file = File::open(file_path)?;
    let er = EventReader::new(file);
    let mut content = String::new();
    for event in er.into_iter() {
        if let XmlEvent::Characters(text) = event.expect("TODO") {
            content.push_str(&text);
            content.push_str(" ");
        }
    }
    Ok(content)
}

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

fn main() -> io::Result<()> {
    let index_path = "index.json";
    let index_file = File::open(index_path)?;
    println!("Reading {index_path} index file...");
    let tf_index: TermFreqIndex = serde_json::from_reader(index_file).expect("serde does not fail");
    println!(
        "{index_path} contains {count} files",
        count = tf_index.len()
    );
    Ok(())
}

fn main2() -> io::Result<()> {
    // let file_path = "docs.gl/gl4/glVertexAttribDivisor.xhtml";

    let dir_path = "docs.gl/gl4";
    let dir = fs::read_dir(dir_path)?;
    let top_n = 15;
    let mut tf_index = TermFreqIndex::new();

    for file in dir {
        let file_path = file?.path();

        println!("Indexing {:?}...", &file_path);

        let content = read_entire_xml_file(&file_path)?
            .chars()
            .collect::<Vec<_>>();

        // let mut tf = HashMap::<String, uszie>::new();
        let mut tf = TermFreq::new();

        for token in Lexer::new(&content) {
            let term = token
                .iter()
                .map(|x| x.to_ascii_uppercase())
                .collect::<String>();

            if let Some(freq) = tf.get_mut(&term) {
                *freq += 1;
            } else {
                tf.insert(term, 1);
            }
        }

        let mut stats = tf.iter().collect::<Vec<_>>();
        stats.sort_by_key(|(_, f)| *f);
        stats.reverse();

        tf_index.insert(file_path, tf);
        // println!("{file_path:?}");
        // for (t, f) in stats.iter().take(top_n) {
        //     println!("   {t} => {f}")
        // }
    }

    let index_path = "index.json";
    println!("Saving {index_path}...");
    let index_file = File::create(index_path)?;
    serde_json::to_writer(&index_file, &tf_index).expect("Serde works fine!");

    // let j = serde_json::to_string(&tf_index)?;

    // for (path, tf) in tf_index {
    //     println!("{path:?} has {count} unique tokens", count = tf.len())
    // }

    Ok(())
}
