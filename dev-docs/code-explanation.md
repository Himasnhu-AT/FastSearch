# Code Documentation

This document provides an overview of the code structure and explanation of the code in the FastSearch project.

## `packages/engine/src/main.rs`:

### structs:

- `Lexer`: This struct is used for lexical analysis of the input. It holds a reference to a slice of characters (content) which it operates on. The struct and its fields are only accessible within the current crate.
  > ```rust
  > pub(crate) struct Lexer<'a> {
  >     pub(crate) content: &'a [char],
  > }
  > ```

---

---

### Functions:

---

###### - `tf(t: &str, d: &TermFreq) -> f32`:

This function calculates the term frequency (tf) of a given term `t` in a document represented by the `TermFreq` data structure. It returns the tf value as a floating-point number. If the term `t` is not found in the `TermFreq` data structure, it returns 0.

---

###### - `idf(t: &str, d: &TermFreqIndex) -> f32`:

This function calculates the inverse document frequency (idf) of a given term `t` in a collection of documents represented by the `TermFreqIndex` data structure. It returns the idf value as a floating-point number. The idf value is calculated based on the total number of documents in the collection (`N`) and the number of documents that contain the term `t` (`M`). If `M` is 0, it is set to 1 to avoid division by zero. The idf value is calculated using the logarithm base 10.

---

###### - `tf_index_of_folder(dir_path: &Path, tf_index: &mut TermFreqIndex) -> Result<(), ()>`:

This function is used to index the contents of a folder and populate a term frequency index (`tf_index`) with the term frequencies of the files in the folder. It takes two parameters: `dir_path`, which is the path to the folder to be indexed, and `tf_index`, which is a mutable reference to the term frequency index.

Arguments:

- `dir_path` - The path to the folder to be indexed.
- `tf_index` - A mutable reference to the term frequency index (`TermFreqIndex`).

---

###### - `save_tf_index(tf_index: &TermFreqIndex, index_path: &str) -> Result<(), ()>`:

This function saves the term frequency index (`tf_index`) to a file specified by `index_path`.
It returns a `Result` indicating whether the operation was successful or not.

Arguments:

- `tf_index` - A reference to the term frequency index (`TermFreqIndex`) to be saved.
- `index_path` - The path to the file where the index will be saved.

---

---

### Type Definitions:

- `TermFreq`:

  > ```rust
  > pub(crate) type TermFreq = HashMap<String, usize>;
  > ```

- `TermFreqIndex`:
  > ```rust
  > pub(crate) type TermFreqIndex = HashMap<PathBuf, HashMap<String, usize>>;
  > ```
